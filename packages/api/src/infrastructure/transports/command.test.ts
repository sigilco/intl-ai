import { describe, it, expect } from "vitest";
import { createCommandTransport, extractJsonPayload } from "./command";

const node = process.execPath;

function runOpts(script: string, extra?: Partial<Parameters<typeof createCommandTransport>[0]>) {
  return createCommandTransport({
    id: "fake-agent",
    command: node,
    args: ["-e", script],
    ...extra,
  });
}

async function complete(transport: ReturnType<typeof createCommandTransport>) {
  return transport.complete({
    systemPrompt: "sys",
    userPrompt: "user",
    signal: new AbortController().signal,
  });
}

describe("createCommandTransport", () => {
  it("returns clean JSON on stdout", async () => {
    const transport = runOpts(
      `process.stdin.resume(); process.stdout.write(JSON.stringify({translations:[{key:"a",translated:"b"}]}))`,
    );
    const { content } = await complete(transport);
    expect(JSON.parse(content)).toEqual({ translations: [{ key: "a", translated: "b" }] });
  });

  it("extracts JSON wrapped in a fenced block", async () => {
    const transport = runOpts(
      `process.stdout.write("here you go:\\n\\\`\\\`\\\`json\\n" + JSON.stringify({translations:[]}) + "\\n\\\`\\\`\\\`\\n")`,
    );
    const { content } = await complete(transport);
    expect(JSON.parse(content)).toEqual({ translations: [] });
  });

  it("extracts JSON embedded in prose", async () => {
    const transport = runOpts(
      `process.stdout.write("Sure, here is the result: " + JSON.stringify({translations:[]}) + " Hope that helps!")`,
    );
    const { content } = await complete(transport);
    expect(JSON.parse(content)).toEqual({ translations: [] });
  });

  it("strips ANSI control characters from output", async () => {
    const transport = runOpts(
      `process.stdout.write("\\u001b[32m" + JSON.stringify({translations:[]}) + "\\u001b[0m")`,
    );
    const { content } = await complete(transport);
    expect(JSON.parse(content)).toEqual({ translations: [] });
  });

  it("ignores noisy stderr when stdout is clean", async () => {
    const transport = runOpts(
      `process.stderr.write("warning: something\\n"); process.stdout.write(JSON.stringify({translations:[]}))`,
    );
    const { content } = await complete(transport);
    expect(JSON.parse(content)).toEqual({ translations: [] });
  });

  it("rejects on non-zero exit", async () => {
    const transport = runOpts(`process.stderr.write("boom"); process.exit(1)`);
    await expect(complete(transport)).rejects.toThrow(/exited with code 1/);
  });

  it("rejects on ENOENT (binary not found)", async () => {
    const transport = createCommandTransport({
      id: "fake-agent",
      command: "definitely-not-a-real-binary-xyz",
      args: [],
    });
    await expect(complete(transport)).rejects.toThrow(/ENOENT/);
  });

  it("kills a process that never exits and rejects with a timeout error", async () => {
    const transport = runOpts(`setInterval(() => {}, 1000)`, { timeoutMs: 100 });
    await expect(complete(transport)).rejects.toThrow(/timed out after 100ms/);
  }, 10_000);

  it("rejects when stdout exceeds the byte cap", async () => {
    const transport = runOpts(`process.stdout.write("x".repeat(1000))`, { maxStdoutBytes: 10 });
    await expect(complete(transport)).rejects.toThrow(/exceeded 10 bytes/);
  });

  it("sends the prompt via argv instead of stdin when promptVia is 'argv'", async () => {
    const transport = runOpts(
      `process.stdout.write(JSON.stringify({translations:[{key:"argv-check",translated:process.argv[1] ?? ""}]}))`,
      { promptVia: "argv" },
    );
    const { content } = await complete(transport);
    const parsed = JSON.parse(content);
    expect(parsed.translations[0].translated).toContain("sys");
    expect(parsed.translations[0].translated).toContain("user");
  });
});

describe("extractJsonPayload", () => {
  it("returns undefined when there is no JSON at all", () => {
    expect(extractJsonPayload("nothing to see here")).toBeUndefined();
  });

  it("prefers a fenced block over inline braces", () => {
    const text = 'prefix {not: the one} ```json\n{"real": true}\n```';
    expect(extractJsonPayload(text)).toBe('{"real": true}');
  });

  it("balances nested braces and strings containing braces", () => {
    const text = 'noise {"a": {"b": "c}"}, "d": 1} trailer';
    expect(extractJsonPayload(text)).toBe('{"a": {"b": "c}"}, "d": 1}');
  });
});
