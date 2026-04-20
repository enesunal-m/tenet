use chrono::NaiveDate;
use serde::Deserialize;

use crate::error::TenetError;

/// Supported priority values in rule frontmatter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Priority {
    /// High-priority rule.
    High,
    /// Normal-priority rule.
    #[default]
    Normal,
    /// Low-priority rule.
    Low,
}

/// Parsed frontmatter values.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Frontmatter {
    /// Scope glob matched against repo-relative paths.
    pub scope: Option<String>,
    /// Optional owner.
    pub owner: Option<String>,
    /// Optional review date.
    pub reviewed: Option<NaiveDate>,
    /// Rule priority.
    pub priority: Priority,
    /// Optional tags.
    pub tags: Vec<String>,
    /// Unknown keys seen in frontmatter.
    pub unknown_fields: Vec<String>,
}

/// Parsed markdown document with optional frontmatter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedRuleDoc {
    /// Parsed frontmatter (or defaults if omitted).
    pub frontmatter: Frontmatter,
    /// Rule body markdown (LF-normalized).
    pub body: String,
    /// Whether the input included a frontmatter block.
    pub had_frontmatter: bool,
}

/// Parse a rule markdown file containing optional YAML frontmatter.
pub fn parse_rule_document(input: &str) -> Result<ParsedRuleDoc, TenetError> {
    let normalized = normalize_input(input);

    if !normalized.starts_with("---\n") {
        return Ok(ParsedRuleDoc {
            frontmatter: Frontmatter::default(),
            body: normalized,
            had_frontmatter: false,
        });
    }

    let remainder = &normalized[4..];
    let (yaml_text, body_start) = if remainder.starts_with("---\n") {
        ("", "---\n".len())
    } else {
        let Some(close_idx) = remainder.find("\n---\n") else {
            return Err(TenetError::BadFrontmatter {
                message: String::from("frontmatter start found without closing delimiter"),
            });
        };

        (&remainder[..close_idx], close_idx + "\n---\n".len())
    };
    let body = remainder[body_start..].to_string();

    let raw: RawFrontmatter = if yaml_text.trim().is_empty() {
        RawFrontmatter::default()
    } else {
        serde_yaml::from_str(yaml_text).map_err(|source| TenetError::BadFrontmatter {
            message: source.to_string(),
        })?
    };

    let priority = parse_priority(raw.priority.as_deref())?;
    let reviewed = parse_reviewed(raw.reviewed.as_deref())?;

    Ok(ParsedRuleDoc {
        frontmatter: Frontmatter {
            scope: raw.scope,
            owner: raw.owner,
            reviewed,
            priority,
            tags: raw.tags.unwrap_or_default(),
            unknown_fields: raw.extra_keys.keys().cloned().collect(),
        },
        body,
        had_frontmatter: true,
    })
}

fn normalize_input(input: &str) -> String {
    let without_bom = input.strip_prefix('\u{FEFF}').unwrap_or(input);
    without_bom.replace("\r\n", "\n")
}

fn parse_priority(value: Option<&str>) -> Result<Priority, TenetError> {
    match value {
        None => Ok(Priority::Normal),
        Some("high") => Ok(Priority::High),
        Some("normal") => Ok(Priority::Normal),
        Some("low") => Ok(Priority::Low),
        Some(other) => Err(TenetError::BadFrontmatterValue {
            field: String::from("priority"),
            message: format!("invalid priority '{other}'"),
        }),
    }
}

fn parse_reviewed(value: Option<&str>) -> Result<Option<NaiveDate>, TenetError> {
    match value {
        None => Ok(None),
        Some(date) => NaiveDate::parse_from_str(date, "%Y-%m-%d")
            .map(Some)
            .map_err(|_| TenetError::BadFrontmatterValue {
                field: String::from("reviewed"),
                message: format!("invalid date '{date}', expected YYYY-MM-DD"),
            }),
    }
}

#[derive(Debug, Default, Deserialize)]
struct RawFrontmatter {
    #[serde(default)]
    scope: Option<String>,
    #[serde(default)]
    owner: Option<String>,
    #[serde(default)]
    reviewed: Option<String>,
    #[serde(default)]
    priority: Option<String>,
    #[serde(default)]
    tags: Option<Vec<String>>,
    #[serde(flatten)]
    extra_keys: std::collections::BTreeMap<String, serde_yaml::Value>,
}

#[cfg(test)]
mod tests {
    use super::{parse_rule_document, Priority};

    #[test]
    fn parses_document_without_frontmatter() {
        let parsed = parse_rule_document("hello\nworld\n").expect("parse");
        assert!(!parsed.had_frontmatter);
        assert_eq!(parsed.body, "hello\nworld\n");
        assert_eq!(parsed.frontmatter.priority, Priority::Normal);
    }

    #[test]
    fn parses_frontmatter_and_body() {
        let parsed = parse_rule_document(
            "---\nscope: \"apps/**\"\npriority: high\nreviewed: 2026-01-01\n---\nBody\n",
        )
        .expect("parse");

        assert!(parsed.had_frontmatter);
        assert_eq!(parsed.frontmatter.scope.as_deref(), Some("apps/**"));
        assert_eq!(parsed.frontmatter.priority, Priority::High);
        assert_eq!(parsed.body, "Body\n");
    }

    #[test]
    fn parses_empty_frontmatter_as_defaults() {
        let parsed = parse_rule_document("---\n---\nBody\n").expect("parse");
        assert!(parsed.had_frontmatter);
        assert_eq!(parsed.frontmatter.priority, Priority::Normal);
        assert_eq!(parsed.body, "Body\n");
    }

    #[test]
    fn rejects_invalid_yaml_frontmatter() {
        let result = parse_rule_document("---\n: nope\n---\nbody\n");
        assert!(result.is_err());
    }

    #[test]
    fn rejects_invalid_priority_value() {
        let result = parse_rule_document("---\npriority: urgent\n---\nbody\n");
        assert!(result.is_err());
    }

    #[test]
    fn normalizes_bom_and_crlf() {
        let parsed = parse_rule_document("\u{FEFF}---\r\npriority: normal\r\n---\r\nBody\r\n")
            .expect("parse");
        assert_eq!(parsed.body, "Body\n");
    }

    #[test]
    fn keeps_unknown_fields_for_linting() {
        let parsed = parse_rule_document("---\ncustom: value\n---\nbody\n").expect("parse");
        assert_eq!(
            parsed.frontmatter.unknown_fields,
            vec![String::from("custom")]
        );
    }
}
