use std::collections::BTreeMap;

use chrono::Local;

use crate::{config, error::TenetError, rule, util::paths::find_repo_root};

pub fn run(grace: Option<u32>, owner: Option<String>, json: bool) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let repo_root = find_repo_root(&cwd)?;

    let cfg = config::load(&repo_root)?.config;
    let grace_days = i64::from(grace.unwrap_or(cfg.defaults.grace_days as u32));
    let today = Local::now().date_naive();

    let rules = rule::load_all(&repo_root)?;

    let mut grouped: BTreeMap<String, Vec<(String, String, String, i64)>> = BTreeMap::new();

    for rule in rules {
        let Some(reviewed) = rule.frontmatter.reviewed else {
            continue;
        };
        let overdue = (today - (reviewed + chrono::Days::new(grace_days as u64))).num_days();
        if overdue <= 0 {
            continue;
        }

        let owner_name = rule
            .frontmatter
            .owner
            .clone()
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| String::from("unowned"));

        if let Some(filter_owner) = owner.as_deref() {
            if owner_name != filter_owner {
                continue;
            }
        }

        grouped.entry(owner_name).or_default().push((
            rule.id,
            type_name(&rule.rule_type).to_string(),
            reviewed.to_string(),
            overdue,
        ));
    }

    if json {
        println!("[");
        let mut first = true;
        for (owner, entries) in &grouped {
            for (id, typ, reviewed, overdue) in entries {
                if !first {
                    println!(",");
                }
                print!(
                    "  {{\"owner\":\"{owner}\",\"id\":\"{id}\",\"type\":\"{typ}\",\"reviewed\":\"{reviewed}\",\"days_overdue\":{overdue}}}"
                );
                first = false;
            }
        }
        println!("\n]");
    } else {
        for (owner_name, entries) in &grouped {
            println!("{owner_name}:");
            for (id, typ, reviewed, overdue) in entries {
                println!("  {id}\t{typ}\t{reviewed}\t{overdue} days overdue");
            }
        }
    }

    if grouped.is_empty() {
        Ok(())
    } else {
        Err(TenetError::StaleRulesFound.into())
    }
}

fn type_name(rule_type: &rule::RuleType) -> &'static str {
    match rule_type {
        rule::RuleType::Invariants => "invariants",
        rule::RuleType::Conventions => "conventions",
        rule::RuleType::Decisions => "decisions",
        rule::RuleType::Gotchas => "gotchas",
        rule::RuleType::Glossary => "glossary",
    }
}
