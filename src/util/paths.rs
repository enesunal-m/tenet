use std::path::{Path, PathBuf};

use crate::error::TenetError;

/// Find repository root by walking up until `.git` is found.
pub fn find_repo_root(start: &Path) -> Result<PathBuf, TenetError> {
    let mut current = start.to_path_buf();

    loop {
        if current.join(".git").exists() {
            return Ok(current);
        }

        if !current.pop() {
            return Err(TenetError::NotInGitRepo);
        }
    }
}
