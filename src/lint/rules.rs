use std::{fs, path::Path};

use walkdir::WalkDir;

use crate::{
    lint::{secrets::SecretPatterns, Finding, Severity},
    rule::frontmatter::parse_rule_document,
};

pub fn collect_findings(repo_root: &Path, check_secrets_override: bool) -> Vec<Finding> {
    let mut findings = Vec::new();
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
                if let Some(scope) = parsed.frontmatter.scope.as_deref() {
                    if scope.starts_with('/') {
                        findings.push(Finding {
                            severity: Severity::Error,
                            file: rel.clone(),
                            line: 1,
                            id: "abs-scope",
                            message: String::from("scope begins with '/'"),
                        });
                    }
                }

                if check_secrets_override || contains_secrets_default_enabled() {
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
            Err(_) => {
                findings.push(Finding {
                    severity: Severity::Error,
                    file: rel,
                    line: 1,
                    id: "bad-frontmatter",
                    message: String::from("frontmatter does not parse as YAML"),
                });
            }
        }
    }

    findings
}

fn contains_secrets_default_enabled() -> bool {
    true
}
