use std::path::{Path, PathBuf};

use chrono::NaiveDate;

use crate::{
    compile::marker::GENERATED_HEADER,
    rule::{Rule, RuleType},
};

pub fn render_agents_md(
    repo_root: &Path,
    anchor: PathBuf,
    rules: &[Rule],
    stale_rules: &[Rule],
    today: NaiveDate,
) -> String {
    let mut out = String::new();
    out.push_str(GENERATED_HEADER);
    out.push('\n');
    out.push_str(&format!("<!-- last compiled: {}T00:00:00Z -->\n", today));
    out.push_str(&format!(
        "<!-- rules: {} -->\n\n",
        rules.len() + stale_rules.len()
    ));
    out.push_str("# Project context\n\n");

    if rules.is_empty() && stale_rules.is_empty() {
        out.push_str("No rules defined.\n");
        return out;
    }

    render_section(&mut out, "Invariants", RuleType::Invariants, rules);
    render_section(&mut out, "Conventions", RuleType::Conventions, rules);
    render_section(&mut out, "Decisions", RuleType::Decisions, rules);
    render_section(&mut out, "Gotchas", RuleType::Gotchas, rules);
    render_section(&mut out, "Glossary", RuleType::Glossary, rules);

    if !stale_rules.is_empty() {
        out.push_str("## Needs review\n\n");
        for rule in stale_rules {
            let lead = rule.body.lines().next().unwrap_or_default().trim();
            out.push_str(&format!(
                "- **{}**\n\n",
                if lead.is_empty() {
                    "(empty rule)"
                } else {
                    lead
                }
            ));
        }
    }

    if anchor.as_os_str().is_empty() {
        out.push_str("{{SEE_ALSO}}");
    } else {
        out.push_str("\n---\n");
        out.push_str(&format!(
            "*Generated from `{}`. Do not edit directly.*\n",
            relative_context_path(repo_root, &anchor)
        ));
    }

    out
}

fn relative_context_path(_repo_root: &Path, anchor: &Path) -> String {
    let depth = anchor.components().count();
    if depth == 0 {
        return String::from(".context/");
    }
    let mut prefix = String::new();
    for _ in 0..depth {
        prefix.push_str("../");
    }
    format!("{}{}.context/", "", prefix)
}

fn render_section(out: &mut String, title: &str, target_type: RuleType, rules: &[Rule]) {
    let filtered: Vec<&Rule> = rules
        .iter()
        .filter(|rule| rule.rule_type == target_type)
        .collect();
    if filtered.is_empty() {
        return;
    }

    out.push_str(&format!("## {title}\n\n"));

    for rule in filtered {
        let mut lines = rule.body.lines();
        let lead = lines
            .next()
            .unwrap_or_default()
            .trim_start_matches('#')
            .trim();
        let lead = if lead.is_empty() {
            "(empty rule)"
        } else {
            lead
        };
        out.push_str(&format!("- **{lead}**  \n"));
        for line in lines {
            out.push_str(&format!("  {line}\n"));
        }

        let mut meta = Vec::new();
        if let Some(scope) = &rule.frontmatter.scope {
            if scope != "**" {
                meta.push(format!("scope: {scope}"));
            }
        }
        if let Some(owner) = &rule.frontmatter.owner {
            if !owner.is_empty() {
                meta.push(format!("owner: {owner}"));
            }
        }
        if let Some(reviewed) = rule.frontmatter.reviewed {
            meta.push(format!("reviewed: {reviewed}"));
        }

        if !meta.is_empty() {
            out.push_str(&format!("\n  *{}*\n", meta.join(" · ")));
        }

        out.push('\n');
    }
}
