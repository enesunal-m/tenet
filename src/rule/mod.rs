use std::{fs, path::Path};

use walkdir::WalkDir;

use crate::error::TenetError;

use self::{frontmatter::parse_rule_document, id::rule_id_from_path};

pub mod frontmatter;
pub mod id;
pub mod scope;

/// Canonical rule type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleType {
    /// Invariant rule.
    Invariants,
    /// Convention rule.
    Conventions,
    /// Decision rule.
    Decisions,
    /// Gotcha rule.
    Gotchas,
    /// Glossary rule.
    Glossary,
}

impl RuleType {
    fn from_dir_name(value: &str) -> Option<Self> {
        match value {
            "invariants" => Some(Self::Invariants),
            "conventions" => Some(Self::Conventions),
            "decisions" => Some(Self::Decisions),
            "gotchas" => Some(Self::Gotchas),
            "glossary" => Some(Self::Glossary),
            _ => None,
        }
    }
}

/// A parsed rule file under `.context/`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    /// Rule ID `<type>/<slug>`.
    pub id: String,
    /// Rule type.
    pub rule_type: RuleType,
    /// Path to the file on disk.
    pub path: std::path::PathBuf,
    /// Parsed frontmatter and metadata.
    pub frontmatter: frontmatter::Frontmatter,
    /// Markdown body.
    pub body: String,
}

/// Load all recognized `.context/<type>/*.md` rules from `repo_root`.
pub fn load_all(repo_root: &Path) -> Result<Vec<Rule>, TenetError> {
    let context_root = repo_root.join(".context");
    if !context_root.exists() {
        return Ok(Vec::new());
    }

    let mut rules = Vec::new();

    for entry in WalkDir::new(&context_root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.into_path();
        if path.extension().and_then(|value| value.to_str()) != Some("md") {
            continue;
        }

        let Some(type_name) = path
            .parent()
            .and_then(|value| value.file_name())
            .and_then(|value| value.to_str())
        else {
            continue;
        };
        let Some(rule_type) = RuleType::from_dir_name(type_name) else {
            continue;
        };

        let raw = fs::read_to_string(&path).map_err(|source| TenetError::Io {
            path: path.display().to_string(),
            source,
        })?;
        let parsed = parse_rule_document(&raw)?;
        let id = rule_id_from_path(&path)?;

        rules.push(Rule {
            id,
            rule_type,
            path,
            frontmatter: parsed.frontmatter,
            body: parsed.body,
        });
    }

    rules.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(rules)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::load_all;

    #[test]
    fn load_all_reads_recognized_rule_files() {
        let temp_dir = TempDir::new().expect("tempdir");
        let root = temp_dir.path();
        fs::create_dir_all(root.join(".context/invariants")).expect("create dir");
        fs::create_dir_all(root.join(".context/random")).expect("create dir");
        fs::write(
            root.join(".context/invariants/session.md"),
            "---\npriority: normal\n---\nSession rules\n",
        )
        .expect("write rule");
        fs::write(root.join(".context/random/ignored.md"), "ignored\n").expect("write rule");

        let rules = load_all(root).expect("load rules");

        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].id, "invariants/session");
        assert_eq!(rules[0].body, "Session rules\n");
    }
}
