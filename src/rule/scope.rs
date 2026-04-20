use std::path::{Path, PathBuf};

use globset::Glob;

use crate::error::TenetError;

/// Result of anchor directory computation for a scope expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchorResult {
    /// Anchor directory path, relative to repo root.
    pub anchor: PathBuf,
    /// Whether a literal anchor path was derived but missing on disk.
    pub missing_dir: bool,
}

/// Check whether `scope` is a valid glob and whether it matches `repo_relative_path`.
pub fn scope_matches(scope: &str, repo_relative_path: &Path) -> Result<bool, TenetError> {
    let scope = validate_scope(scope)?;
    let matcher = Glob::new(scope)
        .map_err(|source| TenetError::InvalidScope {
            scope: scope.to_string(),
            message: source.to_string(),
        })?
        .compile_matcher();

    Ok(matcher.is_match(repo_relative_path))
}

/// Validate a scope string per v0 rules.
pub fn validate_scope(scope: &str) -> Result<&str, TenetError> {
    if scope.starts_with('/') {
        return Err(TenetError::AbsoluteScope {
            scope: scope.to_string(),
        });
    }

    Glob::new(scope).map_err(|source| TenetError::InvalidScope {
        scope: scope.to_string(),
        message: source.to_string(),
    })?;

    Ok(scope)
}

/// Compute the anchor directory for a scope relative to `repo_root`.
pub fn compute_anchor(repo_root: &Path, scope: &str) -> AnchorResult {
    let normalized = normalize_scope_for_anchor(scope);
    let mut components = Vec::new();

    for component in normalized.split('/') {
        if component.is_empty() {
            continue;
        }
        if has_glob_metachar(component) {
            break;
        }
        components.push(component);
    }

    if components.is_empty() {
        return AnchorResult {
            anchor: PathBuf::new(),
            missing_dir: false,
        };
    }

    let anchor = components.iter().collect::<PathBuf>();
    if repo_root.join(&anchor).exists() {
        AnchorResult {
            anchor,
            missing_dir: false,
        }
    } else {
        AnchorResult {
            anchor: PathBuf::new(),
            missing_dir: true,
        }
    }
}

fn normalize_scope_for_anchor(scope: &str) -> String {
    let without_trailing_glob = scope.strip_suffix("/**").unwrap_or(scope);
    without_trailing_glob.trim_end_matches('/').to_string()
}

fn has_glob_metachar(component: &str) -> bool {
    component.contains('*')
        || component.contains('?')
        || component.contains('[')
        || component.contains('{')
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use tempfile::TempDir;

    use super::{compute_anchor, scope_matches, validate_scope};

    #[test]
    fn validate_scope_rejects_absolute_scope() {
        let result = validate_scope("/apps/**");
        assert!(result.is_err());
    }

    #[test]
    fn scope_matching_uses_globset() {
        let matched = scope_matches("apps/bundle/**", Path::new("apps/bundle/src/main.rs"))
            .expect("scope parse");
        assert!(matched);
    }

    #[test]
    fn compute_anchor_examples_from_spec_table() {
        let temp_dir = TempDir::new().expect("tempdir");
        let root = temp_dir.path();

        fs::create_dir_all(root.join("apps/bundle/src/auth")).expect("create dir");
        fs::create_dir_all(root.join("apps/foo/src")).expect("create dir");

        assert_eq!(compute_anchor(root, "**").anchor, Path::new(""));
        assert_eq!(
            compute_anchor(root, "apps/bundle/**").anchor,
            Path::new("apps/bundle")
        );
        assert_eq!(
            compute_anchor(root, "apps/bundle/src/auth/*.rs").anchor,
            Path::new("apps/bundle/src/auth")
        );
        assert_eq!(compute_anchor(root, "**/*.test.ts").anchor, Path::new(""));
        assert_eq!(
            compute_anchor(root, "apps/*/src/**").anchor,
            Path::new("apps")
        );
    }

    #[test]
    fn compute_anchor_falls_back_to_root_if_anchor_missing() {
        let temp_dir = TempDir::new().expect("tempdir");
        let result = compute_anchor(temp_dir.path(), "apps/missing/**");

        assert_eq!(result.anchor, Path::new(""));
        assert!(result.missing_dir);
    }
}
