use std::{collections::BTreeSet, fs};

use walkdir::WalkDir;

use crate::{compile, error::TenetError, lint::Severity, rule, util::paths::find_repo_root};

pub fn run(check_compiled: bool, check_secrets: bool, quiet: bool) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let repo_root = find_repo_root(&cwd)?;

    let mut findings = crate::lint::run(&repo_root, check_secrets);

    if check_compiled && compiled_drift_exists(&repo_root)? {
        findings.push(crate::lint::Finding {
            severity: Severity::Warning,
            file: String::from("AGENTS.md"),
            line: 1,
            id: "compiled-drift",
            message: String::from("generated AGENTS.md files are out of sync"),
        });
    }

    for finding in &findings {
        if !quiet {
            println!(
                "{}: {}:{}: {}: {}",
                finding.severity.as_str(),
                finding.file,
                finding.line,
                finding.id,
                finding.message
            );
        }
    }

    let has_error = findings
        .iter()
        .any(|finding| finding.severity == Severity::Error);
    let has_warning = findings
        .iter()
        .any(|finding| finding.severity == Severity::Warning);

    if has_error {
        Err(TenetError::LintErrors.into())
    } else if has_warning {
        Err(TenetError::LintWarnings.into())
    } else {
        Ok(())
    }
}

fn compiled_drift_exists(repo_root: &std::path::Path) -> anyhow::Result<bool> {
    let rules = rule::load_all(repo_root)?;
    let cfg = crate::config::load(repo_root)?.config;
    let options = compile::plan::PlanOptions {
        include_stale: cfg.compile.include_stale,
        segregate_stale: cfg.compile.segregate_stale,
        grace_days: cfg.defaults.grace_days,
        today: chrono::Local::now().date_naive(),
    };
    let plan = compile::plan::build_plan(repo_root, &rules, options);
    let planned_paths: BTreeSet<_> = plan.iter().map(|entry| entry.path.clone()).collect();

    for planned in &plan {
        match fs::read_to_string(&planned.path) {
            Ok(on_disk) => {
                if on_disk != planned.content {
                    return Ok(true);
                }
            }
            Err(_) => return Ok(true),
        }
    }

    for existing in WalkDir::new(repo_root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.into_path())
        .filter(|path| path.file_name().and_then(|value| value.to_str()) == Some("AGENTS.md"))
    {
        if !planned_paths.contains(&existing)
            && compile::marker::is_generated_agents(&existing).unwrap_or(false)
        {
            return Ok(true);
        }
    }

    Ok(false)
}
