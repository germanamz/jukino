//! The no-blob-comments check.
//!
//! Explanations belong in doc comments (`///`, `//!`). Plain `//` comments are
//! limited to a single line, block comments are not used, and commented-out code is
//! deleted. The source is lexed as Rust, so text inside string and character
//! literals is never mistaken for a comment.

use std::path::Path;

use ra_ap_rustc_lexer::{FrontmatterAllowed, TokenKind, tokenize};

use crate::finding::Finding;

/// Checks one Rust source file and returns its findings, ordered by line.
#[must_use]
pub fn check_source(path: &Path, source: &str) -> Vec<Finding> {
    let mut scan = Scan::default();
    let mut offset = 0;
    for token in tokenize(source, FrontmatterAllowed::No) {
        let end = offset + token.len as usize;
        scan.visit(token.kind, source.get(offset..end).unwrap_or_default());
        offset = end;
    }
    let mut problems = scan.problems;
    problems.extend(blob_problems(&scan.comment_only_lines));
    problems.sort_by_key(|(line, _)| *line);
    problems
        .into_iter()
        .map(|(line, message)| Finding {
            path: path.to_path_buf(),
            line,
            message,
        })
        .collect()
}

/// Walks the token stream, tracking which lines hold nothing but a plain comment.
struct Scan {
    line: usize,
    line_has_code: bool,
    comment_only_lines: Vec<usize>,
    problems: Vec<(usize, String)>,
}

impl Default for Scan {
    fn default() -> Self {
        Self {
            line: 1,
            line_has_code: false,
            comment_only_lines: Vec::new(),
            problems: Vec::new(),
        }
    }
}

impl Scan {
    fn visit(&mut self, kind: TokenKind, text: &str) {
        if kind == TokenKind::Whitespace {
            self.advance_lines(text);
        } else if let TokenKind::LineComment { doc_style: None } = kind {
            self.visit_plain_comment(text);
        } else if let TokenKind::BlockComment { .. } = kind {
            self.problems.push((
                self.line,
                "block comment: use a `///` or `//!` doc comment, or a single `//` line".to_owned(),
            ));
            self.visit_code(text);
        } else {
            self.visit_code(text);
        }
    }

    fn visit_plain_comment(&mut self, text: &str) {
        if !self.line_has_code {
            self.comment_only_lines.push(self.line);
        }
        let content = text.trim_start_matches('/').trim();
        if content.ends_with([';', '{', '}']) {
            self.problems
                .push((self.line, "commented-out code: delete it".to_owned()));
        }
    }

    fn visit_code(&mut self, text: &str) {
        self.line_has_code = true;
        self.line += text.matches('\n').count();
    }

    fn advance_lines(&mut self, text: &str) {
        let newlines = text.matches('\n').count();
        if newlines > 0 {
            self.line += newlines;
            self.line_has_code = false;
        }
    }
}

fn blob_problems(comment_only_lines: &[usize]) -> Vec<(usize, String)> {
    let mut problems = Vec::new();
    let mut run_start = 0;
    for (index, line) in comment_only_lines.iter().enumerate() {
        let continues = index > 0 && comment_only_lines[index - 1] + 1 == *line;
        if !continues {
            run_start = index;
        }
        let next_continues = comment_only_lines.get(index + 1) == Some(&(line + 1));
        let length = index - run_start + 1;
        if !next_continues && length >= 2 {
            problems.push((
                comment_only_lines[run_start],
                format!(
                    "comment block of {length} lines: move the explanation into a doc comment \
                     (`///` or `//!`) or cut it to one line"
                ),
            ));
        }
    }
    problems
}
