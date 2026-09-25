//! Agent preset table, ported verbatim from `transports/presets.ts`
//! (plan 5.1.7 — the args lists are the contract).

use intl_ai_core::config::{AgentPreset, PromptVia};

pub struct Preset {
    pub command: &'static str,
    pub args: &'static [&'static str],
    pub prompt_via: PromptVia,
}

pub fn resolve(preset: AgentPreset) -> Preset {
    match preset {
        AgentPreset::ClaudeCode => Preset {
            command: "claude",
            args: &[
                "-p",
                "--dangerously-skip-permissions",
                "--output-format",
                "text",
            ],
            prompt_via: PromptVia::Stdin,
        },
        AgentPreset::Opencode => Preset {
            command: "opencode",
            args: &["run", "--auto"],
            prompt_via: PromptVia::Stdin,
        },
        AgentPreset::Codex => Preset {
            command: "codex",
            args: &["exec", "--approve-for-me"],
            prompt_via: PromptVia::Stdin,
        },
        AgentPreset::Crush => Preset {
            command: "crush",
            args: &["run", "-q"],
            prompt_via: PromptVia::Argv,
        },
        AgentPreset::Gemini => Preset {
            command: "gemini",
            args: &["-p", "", "-y"],
            prompt_via: PromptVia::Stdin,
        },
    }
}

pub fn preset_id(preset: AgentPreset) -> &'static str {
    match preset {
        AgentPreset::ClaudeCode => "claude-code",
        AgentPreset::Opencode => "opencode",
        AgentPreset::Codex => "codex",
        AgentPreset::Crush => "crush",
        AgentPreset::Gemini => "gemini",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_match_ts_table() {
        assert_eq!(
            resolve(AgentPreset::ClaudeCode).args,
            &[
                "-p",
                "--dangerously-skip-permissions",
                "--output-format",
                "text"
            ]
        );
        assert_eq!(resolve(AgentPreset::Opencode).args, &["run", "--auto"]);
        assert_eq!(
            resolve(AgentPreset::Codex).args,
            &["exec", "--approve-for-me"]
        );
        assert_eq!(resolve(AgentPreset::Crush).args, &["run", "-q"]);
        assert!(matches!(
            resolve(AgentPreset::Crush).prompt_via,
            PromptVia::Argv
        ));
        assert_eq!(resolve(AgentPreset::Gemini).args, &["-p", "", "-y"]);
        assert!(matches!(
            resolve(AgentPreset::Gemini).prompt_via,
            PromptVia::Stdin
        ));
    }
}
