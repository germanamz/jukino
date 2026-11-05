//! The result type every check reports.

use std::fmt;
use std::path::PathBuf;

/// One problem a check found, at a location an editor can open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// The file, as given to the check.
    pub path: PathBuf,
    /// The line, counting from 1.
    pub line: usize,
    /// What is wrong and how to fix it.
    pub message: String,
}

impl fmt::Display for Finding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}:{}: {}",
            self.path.display(),
            self.line,
            self.message
        )
    }
}
