//! Detects AI attribution in commit messages and pull-request titles.
//!
//! Jukino's history credits people, not tools: no AI co-author trailers, session
//! trailers or links, or "generated with" lines.

/// Names that mark a co-author or "generated with" line as AI attribution.
const AI_NAMES: [&str; 8] = [
    "claude",
    "anthropic",
    "copilot",
    "chatgpt",
    "openai",
    "gemini",
    "codex",
    "cursor",
];

/// The lines of `message`, counting from 1, that attribute the work to an AI tool.
#[must_use]
pub fn attribution_lines(message: &str) -> Vec<usize> {
    message
        .lines()
        .enumerate()
        .filter(|(_, line)| is_attribution(line))
        .map(|(index, _)| index + 1)
        .collect()
}

fn is_attribution(line: &str) -> bool {
    let line = line.trim().to_lowercase();
    let names_ai = AI_NAMES.iter().any(|name| line.contains(name));
    (line.starts_with("co-authored-by:") && names_ai)
        || line.starts_with("claude-session:")
        || line.contains("claude.ai/code/session_")
        || (line.contains("generated with") && names_ai)
}
