use std::{fs, os::unix::fs::PermissionsExt, path::Path};

use crate::error::TenetError;

const BEGIN_MARKER: &str = "# BEGIN TENET PRE-COMMIT HOOK";
const END_MARKER: &str = "# END TENET PRE-COMMIT HOOK";

/// Install tenet's pre-commit hook script into `.git/hooks/pre-commit`.
pub fn install_pre_commit_hook(repo_root: &Path) -> Result<(), TenetError> {
    let hook_path = repo_root.join(".git/hooks/pre-commit");
    let payload = format!(
        "{BEGIN_MARKER}\n{}\n{END_MARKER}\n",
        include_str!("pre_commit.sh").trim_end()
    );

    let content = if hook_path.exists() {
        let existing = fs::read_to_string(&hook_path).map_err(|source| TenetError::Io {
            path: hook_path.display().to_string(),
            source,
        })?;

        if existing.contains(BEGIN_MARKER) && existing.contains(END_MARKER) {
            existing
        } else if existing.trim().is_empty() {
            payload.clone()
        } else {
            format!("{}\n\n{}", existing.trim_end(), payload)
        }
    } else {
        payload.clone()
    };

    fs::write(&hook_path, content).map_err(|source| TenetError::Io {
        path: hook_path.display().to_string(),
        source,
    })?;

    let permissions = fs::Permissions::from_mode(0o755);
    fs::set_permissions(&hook_path, permissions).map_err(|source| TenetError::Io {
        path: hook_path.display().to_string(),
        source,
    })?;

    Ok(())
}
