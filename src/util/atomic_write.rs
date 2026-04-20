use std::{fs, path::Path};

use crate::error::TenetError;

/// Atomically write `content` to `path` using write-then-rename.
pub fn atomic_write(path: &Path, content: &str) -> Result<(), TenetError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| TenetError::Io {
            path: parent.display().to_string(),
            source,
        })?;
    }

    let tmp_path = path.with_extension("md.tmp");
    fs::write(&tmp_path, content).map_err(|source| TenetError::Io {
        path: tmp_path.display().to_string(),
        source,
    })?;
    fs::rename(&tmp_path, path).map_err(|source| TenetError::Io {
        path: path.display().to_string(),
        source,
    })?;

    Ok(())
}
