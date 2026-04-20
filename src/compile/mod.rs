use std::{collections::BTreeSet, fs, path::Path};

use chrono::Local;
use walkdir::WalkDir;

use crate::{config, error::TenetError, rule, util::atomic_write::atomic_write};

use self::marker::is_generated_agents;

pub mod marker;
pub mod plan;
pub mod render;

pub fn compile_repo(
    repo_root: &Path,
    dry_run: bool,
    exclude_stale: bool,
) -> Result<Vec<std::path::PathBuf>, TenetError> {
    let mut rules = Vec::new();
    let mut invalid_count = 0;
    for item in rule::load_all_lenient(repo_root)? {
        match item {
            rule::RuleLoad::Valid(rule) => rules.push(rule),
            rule::RuleLoad::Invalid { path, error } => {
                invalid_count += 1;
                eprintln!("error: skipping invalid rule {}: {error}", path.display());
            }
        }
    }
    if invalid_count > 0 {
        return Err(TenetError::InvalidRules {
            count: invalid_count,
        });
    }
    let cfg = config::load(repo_root)?.config;
    let options = plan::PlanOptions {
        include_stale: if exclude_stale {
            false
        } else {
            cfg.compile.include_stale
        },
        segregate_stale: cfg.compile.segregate_stale,
        grace_days: cfg.defaults.grace_days,
        today: Local::now().date_naive(),
    };
    let plan = plan::build_plan(repo_root, &rules, options);

    let planned_paths: BTreeSet<std::path::PathBuf> =
        plan.iter().map(|entry| entry.path.clone()).collect();

    let mut conflicts = Vec::new();
    let mut stale_generated = Vec::new();

    for existing in find_agents_files(repo_root) {
        let is_target = planned_paths.contains(&existing);
        let generated = is_generated_agents(&existing)?;

        if is_target && !generated {
            conflicts.push(existing);
            continue;
        }

        if !is_target && generated {
            stale_generated.push(existing);
        }
    }

    if !conflicts.is_empty() {
        conflicts.sort();
        return Err(TenetError::HandwrittenConflict { paths: conflicts });
    }

    if dry_run {
        for entry in &plan {
            println!("would write {}", entry.path.display());
            for line in entry.content.lines().take(10) {
                println!("  {line}");
            }
        }
        for path in &stale_generated {
            println!("would delete {}", path.display());
        }
        return Ok(Vec::new());
    }

    for path in stale_generated {
        if path.exists() {
            fs::remove_file(&path).map_err(|source| TenetError::Io {
                path: path.display().to_string(),
                source,
            })?;
        }
    }

    let mut written = Vec::new();
    for entry in plan {
        atomic_write(&entry.path, &entry.content)?;
        written.push(entry.path);
    }

    Ok(written)
}

fn find_agents_files(repo_root: &Path) -> Vec<std::path::PathBuf> {
    WalkDir::new(repo_root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.into_path())
        .filter(|path| path.file_name().and_then(|value| value.to_str()) == Some("AGENTS.md"))
        .collect()
}
