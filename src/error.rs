/// Error type for tenet failures.
#[derive(Debug, thiserror::Error)]
pub enum TenetError {
    /// Filesystem read/write failure.
    #[error("i/o error at {path}: {source}")]
    Io {
        /// Path being accessed.
        path: String,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// Current working directory is not inside a git repository.
    #[error("not inside a git repo")]
    NotInGitRepo,
    /// `.context/` already exists and init was run without `--force`.
    #[error(".context/ already exists in this repo: {path}")]
    AlreadyInitialized {
        /// Existing `.context` directory path.
        path: std::path::PathBuf,
    },
    /// Invalid `.tenetrc` content.
    #[error("invalid config at {path}: {message}")]
    BadConfig {
        /// Path to `.tenetrc`.
        path: String,
        /// Parse/type message.
        message: String,
    },
    /// Invalid YAML frontmatter block.
    #[error("bad frontmatter: {message}")]
    BadFrontmatter {
        /// Parse failure detail.
        message: String,
    },
    /// Invalid frontmatter field value.
    #[error("invalid frontmatter field '{field}': {message}")]
    BadFrontmatterValue {
        /// Field name.
        field: String,
        /// Validation failure detail.
        message: String,
    },
    /// Invalid scope glob syntax.
    #[error("invalid scope '{scope}': {message}")]
    InvalidScope {
        /// Scope string that failed validation.
        scope: String,
        /// Parse failure detail.
        message: String,
    },
    /// Absolute scopes are not allowed.
    #[error("scope must be repo-relative, found absolute scope: {scope}")]
    AbsoluteScope {
        /// Absolute scope string.
        scope: String,
    },
    /// Compile would overwrite hand-written AGENTS.md files.
    #[error("compile conflict with hand-written AGENTS.md: {paths:?}")]
    HandwrittenConflict {
        /// Conflicting paths.
        paths: Vec<std::path::PathBuf>,
    },
    /// Stale rules were detected.
    #[error("stale rules found")]
    StaleRulesFound,
    /// Lint produced warnings.
    #[error("lint warnings")]
    LintWarnings,
    /// Lint produced errors.
    #[error("lint errors")]
    LintErrors,
    /// Invalid rule path-to-id conversion.
    #[error("invalid rule path {path}: {message}")]
    RulePath {
        /// Path to rule.
        path: String,
        /// Validation detail.
        message: String,
    },
}
