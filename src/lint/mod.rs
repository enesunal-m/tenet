use std::path::Path;

pub mod rules;
pub mod secrets;

/// Lint finding severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Non-blocking warning.
    Warning,
    /// Blocking error.
    Error,
}

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Warning => "warning",
            Severity::Error => "error",
        }
    }
}

/// A single lint finding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub severity: Severity,
    pub file: String,
    pub line: usize,
    pub id: &'static str,
    pub message: String,
}

/// Run lint checks and return findings.
pub fn run(repo_root: &Path, check_secrets_override: bool) -> Vec<Finding> {
    rules::collect_findings(repo_root, check_secrets_override)
}
