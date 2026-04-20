use std::path::{Path, PathBuf};

use crate::error::TenetError;

/// Convert a rule file path into its stable rule ID `<type>/<slug>`.
pub fn rule_id_from_path(path: &Path) -> Result<String, TenetError> {
    let parent = path.parent().ok_or_else(|| TenetError::RulePath {
        path: path.display().to_string(),
        message: String::from("missing parent directory"),
    })?;
    let rule_type = parent
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| TenetError::RulePath {
            path: path.display().to_string(),
            message: String::from("invalid type directory"),
        })?;

    let slug = path
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| TenetError::RulePath {
            path: path.display().to_string(),
            message: String::from("invalid file stem"),
        })?;

    Ok(format!("{rule_type}/{slug}"))
}

/// Convert rule ID `<type>/<slug>` back to a filesystem path under `.context/`.
pub fn path_from_rule_id(repo_root: &Path, rule_id: &str) -> Result<PathBuf, TenetError> {
    let (rule_type, slug) = rule_id
        .split_once('/')
        .ok_or_else(|| TenetError::RulePath {
            path: rule_id.to_string(),
            message: String::from("expected <type>/<slug>"),
        })?;

    Ok(repo_root
        .join(".context")
        .join(rule_type)
        .join(format!("{slug}.md")))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{path_from_rule_id, rule_id_from_path};

    #[test]
    fn builds_rule_id_from_context_path() {
        let path = Path::new(".context/invariants/auth-session.md");
        let id = rule_id_from_path(path).expect("id");
        assert_eq!(id, "invariants/auth-session");
    }

    #[test]
    fn builds_path_from_rule_id() {
        let path = path_from_rule_id(Path::new("/repo"), "invariants/auth-session").expect("path");
        assert_eq!(path, Path::new("/repo/.context/invariants/auth-session.md"));
    }
}
