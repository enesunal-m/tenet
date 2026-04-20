use std::{fs, path::Path};

use walkdir::WalkDir;

use crate::{
    config::{self, LintConfig},
    error::TenetError,
    lint::{secrets::SecretPatterns, Finding, Severity},
    rule::frontmatter::parse_rule_document,
};

pub fn collect_findings(repo_root: &Path, check_secrets_override: bool) -> Vec<Finding> {
    let mut findings = Vec::new();
    let lint_config = match config::load(repo_root) {
        Ok(loaded) => {
            for warning in loaded.warnings {
                findings.push(Finding {
                    severity: Severity::Warning,
                    file: String::from(".tenetrc"),
                    line: 1,
                    id: "unknown-config-key",
                    message: warning,
                });
            }
            loaded.config.lint
        }
        Err(error) => {
            findings.push(Finding {
                severity: Severity::Error,
                file: String::from(".tenetrc"),
                line: 1,
                id: "bad-config",
                message: error.to_string(),
            });
            LintConfig::default()
        }
    };
    let patterns = SecretPatterns::compile();

    let context_root = repo_root.join(".context");
    if !context_root.exists() {
        return findings;
    }

    for entry in WalkDir::new(&context_root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.path();

        if path.extension().and_then(|value| value.to_str()) != Some("md") {
            continue;
        }

        let rel = path
            .strip_prefix(repo_root)
            .ok()
            .unwrap_or(path)
            .display()
            .to_string();

        let type_name = path
            .parent()
            .and_then(|value| value.file_name())
            .and_then(|value| value.to_str())
            .unwrap_or("");

        if !matches!(
            type_name,
            "invariants" | "conventions" | "decisions" | "gotchas" | "glossary"
        ) {
            findings.push(Finding {
                severity: Severity::Warning,
                file: rel.clone(),
                line: 1,
                id: "unknown-type-dir",
                message: String::from("subdirectory of .context is not a recognized rule type"),
            });
            continue;
        }

        let content = match fs::read_to_string(path) {
            Ok(value) => value,
            Err(_) => continue,
        };

        match parse_rule_document(&content) {
            Ok(parsed) => {
                if lint_config.check_filenames && !is_kebab_markdown_filename(path) {
                    findings.push(Finding {
                        severity: Severity::Warning,
                        file: rel.clone(),
                        line: 1,
                        id: "bad-filename",
                        message: String::from("filename is not lowercase kebab-case"),
                    });
                }

                for field in parsed.frontmatter.unknown_fields {
                    findings.push(Finding {
                        severity: Severity::Warning,
                        file: rel.clone(),
                        line: 1,
                        id: "unknown-field",
                        message: format!("frontmatter contains unknown field '{field}'"),
                    });
                }

                if parsed.body.trim().is_empty() {
                    findings.push(Finding {
                        severity: Severity::Warning,
                        file: rel.clone(),
                        line: 1,
                        id: "empty-body",
                        message: String::from("rule body is empty"),
                    });
                }

                if let Some(scope) = parsed.frontmatter.scope.as_deref() {
                    match crate::rule::scope::validate_scope(scope) {
                        Ok(_) => {
                            if crate::rule::scope::compute_anchor(repo_root, scope).missing_dir {
                                findings.push(Finding {
                                    severity: Severity::Warning,
                                    file: rel.clone(),
                                    line: 1,
                                    id: "missing-dir",
                                    message: String::from(
                                        "scope references a directory that does not exist",
                                    ),
                                });
                            }
                        }
                        Err(TenetError::AbsoluteScope { .. }) => {
                            findings.push(Finding {
                                severity: Severity::Error,
                                file: rel.clone(),
                                line: 1,
                                id: "abs-scope",
                                message: String::from("scope begins with '/'"),
                            });
                        }
                        Err(TenetError::InvalidScope { message, .. }) => {
                            findings.push(Finding {
                                severity: Severity::Error,
                                file: rel.clone(),
                                line: 1,
                                id: "bad-scope",
                                message,
                            });
                        }
                        Err(_) => {}
                    }
                }

                if check_secrets_override || lint_config.check_secrets {
                    if patterns.github.is_match(&parsed.body) {
                        findings.push(Finding {
                            severity: Severity::Warning,
                            file: rel.clone(),
                            line: 1,
                            id: "secret-github",
                            message: String::from("possible GitHub token detected"),
                        });
                    }
                    if patterns.aws.is_match(&parsed.body) {
                        findings.push(Finding {
                            severity: Severity::Warning,
                            file: rel.clone(),
                            line: 1,
                            id: "secret-aws",
                            message: String::from("possible AWS key detected"),
                        });
                    }
                    if patterns.pem.is_match(&parsed.body) {
                        findings.push(Finding {
                            severity: Severity::Warning,
                            file: rel.clone(),
                            line: 1,
                            id: "secret-pem",
                            message: String::from("possible private key detected"),
                        });
                    }
                }
            }
            Err(error) => findings.push(parse_error_finding(rel, error)),
        }
    }

    findings
}

fn parse_error_finding(file: String, error: TenetError) -> Finding {
    let (id, message) = match error {
        TenetError::BadFrontmatterValue { field, message } if field == "reviewed" => {
            ("bad-date", message)
        }
        TenetError::BadFrontmatterValue { field, message } if field == "priority" => {
            ("bad-priority", message)
        }
        TenetError::BadFrontmatter { message } => (
            "bad-frontmatter",
            format!("frontmatter does not parse as YAML: {message}"),
        ),
        other => ("bad-frontmatter", other.to_string()),
    };

    Finding {
        severity: Severity::Error,
        file,
        line: 1,
        id,
        message,
    }
}

fn is_kebab_markdown_filename(path: &Path) -> bool {
    let Some(stem) = path.file_stem().and_then(|value| value.to_str()) else {
        return false;
    };

    let mut previous_hyphen = true;
    for ch in stem.chars() {
        if ch.is_ascii_lowercase() || ch.is_ascii_digit() {
            previous_hyphen = false;
        } else if ch == '-' && !previous_hyphen {
            previous_hyphen = true;
        } else {
            return false;
        }
    }

    !stem.is_empty() && !previous_hyphen
}
