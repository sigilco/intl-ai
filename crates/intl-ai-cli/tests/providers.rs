//! W1 provider e2e: command transport against a fake agent script, and
//! the OpenAI-compatible transport against a TcpListener mock that
//! captures the wire request. No network is ever touched.

use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::sync::mpsc::{Receiver, channel};
use std::thread;
use tempfile::TempDir;

fn write(dir: &Path, rel: &str, body: &str) {
    let path = dir.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, body).unwrap();
}

fn cmd(dir: &TempDir) -> Command {
    let mut c = Command::cargo_bin("intl-ai").unwrap();
    c.current_dir(dir.path())
        .env("INTL_AI_NOW", "2026-09-25T00:00:00Z");
    c
}

fn fixture(dir: &TempDir, config: &str) {
    write(dir.path(), "intl-ai.toml", config);
    write(
        dir.path(),
        "locales/en.json",
        r#"{"greeting": "Hello", "nav": {"home": "Home"}}"#,
    );
}

fn read_json(dir: &TempDir, rel: &str) -> Value {
    serde_json::from_str(&fs::read_to_string(dir.path().join(rel)).unwrap()).unwrap()
}

// ---------------------------------------------------------------- command

const FAKE_AGENT: &str = r#"#!/bin/sh
# Reads the framed prompt on stdin, records it, answers JSON.
cat > "$AGENT_PROMPT_FILE"
cat <<'EOF'
{"translations":[{"key":"greeting","translated":"Bonjour"},{"key":"nav.home","translated":"Accueil"}]}
EOF
"#;

#[test]
fn command_transport_stdin_framing_and_parse() {
    let dir = TempDir::new().unwrap();
    write(dir.path(), "fake-agent.sh", FAKE_AGENT);
    fixture(
        &dir,
        r#"locale_dir = "locales"
source = "en"
targets = ["fr"]

[provider]
kind = "command"
command = "sh"
args = ["fake-agent.sh"]
"#,
    );
    let prompt_file = dir.path().join("captured.txt");
    cmd(&dir)
        .arg("fill")
        .env("AGENT_PROMPT_FILE", &prompt_file)
        .assert()
        .success();

    let prompt = fs::read_to_string(&prompt_file).unwrap();
    // Frozen framing: system \n\n---\n\n user.
    assert!(prompt.starts_with("You are a professional translation engine."));
    assert!(prompt.contains("\n\n---\n\n"));
    assert!(prompt.contains("Translate the following en strings to fr."));
    assert!(prompt.contains("1. \"Hello\" (key: greeting)"));

    let fr = read_json(&dir, "locales/fr.json");
    assert_eq!(fr["greeting"], "Bonjour");
    assert_eq!(fr["nav"]["home"], "Accueil");
    let lock = fs::read_to_string(dir.path().join("locales/intl-ai.lock.d/fr.toml")).unwrap();
    assert!(lock.contains("model = \"sh\""));
}

#[test]
fn command_transport_argv_prompt_via() {
    let dir = TempDir::new().unwrap();
    write(
        dir.path(),
        "fake-agent.sh",
        r#"#!/bin/sh
# argv prompt: the framed prompt lands as the last positional arg ($1).
printf '%s' "$1" > argv-prompt.txt
printf '%s' '{"translations":[{"key":"greeting","translated":"Salut"}]}'
"#,
    );
    fixture(
        &dir,
        r#"locale_dir = "locales"
source = "en"
targets = ["fr"]

[provider]
kind = "command"
command = "sh"
args = ["fake-agent.sh"]
prompt_via = "argv"
"#,
    );
    cmd(&dir).arg("fill").assert().success();

    let prompt = fs::read_to_string(dir.path().join("argv-prompt.txt")).unwrap();
    assert!(prompt.contains("You are a professional translation engine"));
    let fr = read_json(&dir, "locales/fr.json");
    assert_eq!(fr["greeting"], "Salut");
    // nav.home was omitted from the provider answer -> reported omitted,
    // still a successful run (exit 0, key counted as omitted).
    let shard = intl_ai_core::lockfile::load_shard(&dir.path().join("locales"), "fr").unwrap();
    assert_eq!(
        shard.entries["greeting"].origin,
        intl_ai_core::lockfile::Origin::Ai
    );
}

#[test]
fn command_transport_retries_on_garbage() {
    let dir = TempDir::new().unwrap();
    write(
        dir.path(),
        "flaky-agent.sh",
        r#"#!/bin/sh
# Fails once (plain text, no JSON), then answers.
cat > /dev/null
if [ -f tries.txt ]; then
  cat <<'EOF'
{"translations":[{"key":"greeting","translated":"Bonjour"}]}
EOF
else
  touch tries.txt
  echo "unexpected plain output"
fi
"#,
    );
    fixture(
        &dir,
        r#"locale_dir = "locales"
source = "en"
targets = ["fr"]

[provider]
kind = "command"
command = "sh"
args = ["flaky-agent.sh"]
"#,
    );
    cmd(&dir).arg("fill").assert().success();
    assert!(dir.path().join("tries.txt").exists());
    let fr = read_json(&dir, "locales/fr.json");
    assert_eq!(fr["greeting"], "Bonjour");
}

#[test]
fn command_transport_batch_size_chunks_requests() {
    let dir = TempDir::new().unwrap();
    write(
        dir.path(),
        "counting-agent.sh",
        r#"#!/bin/sh
cat > /dev/null
N=$(cat n.txt 2>/dev/null || echo 0)
echo $((N + 1)) > n.txt
# Over-answers both keys on every call; fill must only take the keys it
# asked for in this batch.
cat <<'EOF'
{"translations":[{"key":"greeting","translated":"Bonjour"},{"key":"nav.home","translated":"Accueil"}]}
EOF
"#,
    );
    fixture(
        &dir,
        r#"locale_dir = "locales"
source = "en"
targets = ["fr"]
batch_size = 1

[provider]
kind = "command"
command = "sh"
args = ["counting-agent.sh"]
"#,
    );
    cmd(&dir).arg("fill").assert().success();
    // Two keys + batch_size 1 -> exactly two transport invocations.
    assert_eq!(
        fs::read_to_string(dir.path().join("n.txt")).unwrap().trim(),
        "2"
    );
}

// ---------------------------------------------------------------- http

struct MockServer {
    url: String,
    requests: Receiver<String>,
}

/// Accepts connections forever; each request text goes into `requests`
/// and gets answered by `respond` (called once per request, in order).
fn mock_server(respond: impl Fn(&str) -> String + Send + 'static) -> MockServer {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let (tx, rx) = channel::<String>();
    thread::spawn(move || {
        for stream in listener.incoming() {
            let mut s = match stream {
                Ok(s) => s,
                Err(_) => continue,
            };
            let mut reader = BufReader::new(s.try_clone().unwrap());
            let mut content_length = 0usize;
            let mut head = String::new();
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).unwrap_or(0) == 0 {
                    break;
                }
                let l = line.trim_end();
                if l.to_ascii_lowercase().starts_with("content-length:") {
                    content_length = l[15..].trim().parse().unwrap_or(0);
                }
                head.push_str(&line);
                if l.is_empty() {
                    break;
                }
            }
            let mut body = vec![0u8; content_length];
            reader.read_exact(&mut body).ok();
            let request = format!("{head}{}", String::from_utf8_lossy(&body));
            tx.send(request.clone()).ok();
            let resp = respond(&request);
            s.write_all(resp.as_bytes()).ok();
        }
    });
    MockServer {
        url: format!("http://127.0.0.1:{port}"),
        requests: rx,
    }
}

fn http_ok(json_content: &str) -> String {
    let body = serde_json::json!({
        "choices": [{ "message": { "content": json_content } }]
    })
    .to_string();
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

fn http_status(status: u16, body: &str) -> String {
    format!(
        "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

#[test]
fn http_transport_wire_shape_and_fence_strip() {
    let dir = TempDir::new().unwrap();
    let srv = mock_server(|_| {
        http_ok(
            "```json\n{\"translations\":[{\"key\":\"greeting\",\"translated\":\"Hola\"},{\"key\":\"nav.home\",\"translated\":\"Inicio\"}]}\n```",
        )
    });
    fixture(
        &dir,
        &format!(
            r#"locale_dir = "locales"
source = "en"
targets = ["fr"]

[provider]
kind = "http"
model = "gpt-mock"
api_key = "sk-test"
base_url = "{}"

[provider.model_params]
temperature = 0.9
"#,
            srv.url
        ),
    );
    cmd(&dir).arg("fill").assert().success();

    let fr = read_json(&dir, "locales/fr.json");
    assert_eq!(fr["greeting"], "Hola");
    assert_eq!(fr["nav"]["home"], "Inicio");

    // Assert the wire request (headers are case-insensitive).
    let req = srv.requests.recv().unwrap();
    let lower = req.to_ascii_lowercase();
    assert!(req.starts_with("POST /chat/completions"), "{req}");
    assert!(lower.contains("authorization: bearer sk-test"), "{req}");
    let body: Value = serde_json::from_str(req.split("\r\n\r\n").nth(1).unwrap_or("{}")).unwrap();
    assert_eq!(body["model"], "gpt-mock");
    assert_eq!(body["messages"][0]["role"], "system");
    assert_eq!(body["response_format"]["type"], "json_schema");
    assert_eq!(
        body["response_format"]["json_schema"]["schema"]["required"],
        serde_json::json!(["translations"])
    );
    // model_params spread last wins over our temperature default.
    assert_eq!(body["temperature"], 0.9);
    // One request total (both keys in a single default batch).
    assert!(srv.requests.try_recv().is_err());
}

#[test]
fn http_transport_retries_429_then_succeeds() {
    let dir = TempDir::new().unwrap();
    let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let calls2 = calls.clone();
    let srv = mock_server(move |_| {
        let n = calls2.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if n == 0 {
            http_status(429, "rate limited")
        } else {
            http_ok("{\"translations\":[{\"key\":\"greeting\",\"translated\":\"Hola\"}]}")
        }
    });
    fixture(
        &dir,
        &format!(
            r#"locale_dir = "locales"
source = "en"
targets = ["fr"]
max_retries = 2

[provider]
kind = "http"
model = "gpt-mock"
api_key = "sk-test"
base_url = "{}"
"#,
            srv.url
        ),
    );
    cmd(&dir).arg("fill").assert().success();
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 2);
}
