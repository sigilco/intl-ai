//! Response-content extraction, ported verbatim from the TS command
//! transport (plan 5.1.8): agent CLIs wrap output in banners, progress
//! spinners, and ANSI/VT escapes; the JSON contract lives inside.

/// Strips ANSI/VT control sequences — CSI (`ESC [` … final), OSC
/// (`ESC ]` … `BEL`/`ESC \`), and simple two-byte escapes. Covers what
/// `util.stripVTControlCharacters` removed in TS.
pub fn strip_vt(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\u{1b}' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('[') => {
                // CSI: parameter/intermediate bytes then a final byte in
                // @ ~.
                for b in chars.by_ref() {
                    if ('@'..='~').contains(&b) {
                        break;
                    }
                }
            }
            Some(']') => {
                // OSC: everything until BEL or ST (ESC \).
                let mut prev = '\0';
                for b in chars.by_ref() {
                    if b == '\u{7}' || (prev == '\u{1b}' && b == '\\') {
                        break;
                    }
                    prev = b;
                }
            }
            Some(_) => {} // two-byte escape: already consumed
            None => {}
        }
    }
    out
}

/// Fenced-block-first, then balanced-brace (string-aware) extraction —
/// same order as `extractJsonPayload` in TS.
pub fn extract_json_payload(text: &str) -> Option<String> {
    // 1. First ```(?:json) ... ``` fenced block (an empty capture is kept,
    //    matching the TS regex's `*?` group).
    if let Some(start) = text.find("```") {
        let after = &text[start + 3..];
        let after = after.strip_prefix("json").unwrap_or(after);
        let after = after.trim_start_matches(|c: char| c.is_whitespace());
        if let Some(end) = after.find("```") {
            return Some(after[..end].trim().to_string());
        }
    }
    // 2. First balanced {...} with string-awareness
    let start = text.find('{')?;
    let bytes = text.as_bytes();
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    let mut i = start;
    while i < bytes.len() {
        let b = bytes[i];
        if in_string {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                in_string = false;
            }
        } else {
            match b {
                b'"' => in_string = true,
                b'{' => depth += 1,
                b'}' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return Some(text[start..=i].to_string());
                    }
                }
                _ => {}
            }
        }
        i += 1;
    }
    None
}

/// What the caller hands `serde_json` after cleanup: the fenced/balanced
/// payload when present, otherwise the de-ANSI'd content itself (TS
/// `extractJsonPayload(cleaned) ?? cleaned`).
pub fn payload_or_self(text: &str) -> String {
    extract_json_payload(text).unwrap_or_else(|| text.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_ansi() {
        let out = strip_vt("\u{1b}[32mok\u{1b}[0m plain");
        assert_eq!(out, "ok plain");
    }

    #[test]
    fn strips_osc() {
        let out = strip_vt("a\u{1b}]8;;http://x\u{7}link\u{1b}]8;;\u{7}b");
        assert_eq!(out, "alinkb");
    }

    #[test]
    fn fenced_json_wins() {
        let text = "banner\n```json\n{\"a\":1}\n```\ntrailer";
        assert_eq!(extract_json_payload(text).unwrap(), "{\"a\":1}");
    }

    #[test]
    fn balanced_brace_fallback() {
        let text = "log line {\"a\":{\"b\":\"}\"}} done";
        assert_eq!(extract_json_payload(text).unwrap(), "{\"a\":{\"b\":\"}\"}}");
    }

    #[test]
    fn no_json_returns_none() {
        assert!(extract_json_payload("no braces here").is_none());
    }
}
