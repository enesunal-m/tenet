use chrono::Local;

use crate::{
    cli::RuleType as CliRuleType,
    config,
    rule::{self, scope::scope_matches, Rule, RuleType},
    util::paths::find_repo_root,
};

pub fn run(
    rule_type: Option<CliRuleType>,
    scope: Option<String>,
    owner: Option<String>,
    stale: bool,
    json: bool,
) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let repo_root = find_repo_root(&cwd)?;

    let config = config::load(&repo_root)?.config;
    let today = Local::now().date_naive();

    let mut rules = rule::load_all(&repo_root)?;
    rules.retain(|rule| {
        type_match(rule, rule_type.as_ref())
            && owner_match(rule, owner.as_deref())
            && scope_match(rule, scope.as_deref())
            && (!stale || is_stale(rule, config.defaults.grace_days, today))
    });

    if json {
        println!("[");
        for (idx, item) in rules.iter().enumerate() {
            let comma = if idx + 1 == rules.len() { "" } else { "," };
            let scope_val = item
                .frontmatter
                .scope
                .clone()
                .unwrap_or_else(|| String::from("**"));
            let anchor = crate::rule::scope::compute_anchor(&repo_root, &scope_val).anchor;
            let is_stale = is_stale(item, config.defaults.grace_days, today);
            println!(
                "  {{\"id\":\"{}\",\"type\":\"{}\",\"scope\":\"{}\",\"owner\":\"{}\",\"reviewed\":\"{}\",\"is_stale\":{},\"anchor\":\"{}\"}}{comma}",
                item.id,
                type_name(&item.rule_type),
                escape_json(&scope_val),
                escape_json(item.frontmatter.owner.as_deref().unwrap_or("")),
                item.frontmatter
                    .reviewed
                    .map(|d| d.to_string())
                    .unwrap_or_default(),
                if is_stale { "true" } else { "false" },
                escape_json(&anchor.display().to_string()),
            );
        }
        println!("]");
        return Ok(());
    }

    println!("ID\tTYPE\tSCOPE\tOWNER\tREVIEWED");
    for rule in rules {
        let scope = rule.frontmatter.scope.unwrap_or_else(|| String::from("**"));
        let owner = rule.frontmatter.owner.unwrap_or_default();
        let reviewed = rule
            .frontmatter
            .reviewed
            .map(|date| date.to_string())
            .unwrap_or_default();
        println!(
            "{}\t{}\t{}\t{}\t{}",
            rule.id,
            type_name(&rule.rule_type),
            scope,
            owner,
            reviewed
        );
    }

    Ok(())
}

fn type_match(rule: &Rule, filter: Option<&CliRuleType>) -> bool {
    match filter {
        None => true,
        Some(value) => match value {
            CliRuleType::Invariants => rule.rule_type == RuleType::Invariants,
            CliRuleType::Conventions => rule.rule_type == RuleType::Conventions,
            CliRuleType::Decisions => rule.rule_type == RuleType::Decisions,
            CliRuleType::Gotchas => rule.rule_type == RuleType::Gotchas,
            CliRuleType::Glossary => rule.rule_type == RuleType::Glossary,
        },
    }
}

fn owner_match(rule: &Rule, filter: Option<&str>) -> bool {
    match filter {
        None => true,
        Some(owner) => rule.frontmatter.owner.as_deref() == Some(owner),
    }
}

fn scope_match(rule: &Rule, path: Option<&str>) -> bool {
    match path {
        None => true,
        Some(path) => {
            let scope = rule.frontmatter.scope.as_deref().unwrap_or("**");
            scope_matches(scope, std::path::Path::new(path)).unwrap_or(false)
        }
    }
}

fn is_stale(rule: &Rule, grace_days: i64, today: chrono::NaiveDate) -> bool {
    rule.frontmatter
        .reviewed
        .map(|reviewed| reviewed + chrono::Days::new(grace_days as u64) < today)
        .unwrap_or(false)
}

fn type_name(rule_type: &RuleType) -> &'static str {
    match rule_type {
        RuleType::Invariants => "invariants",
        RuleType::Conventions => "conventions",
        RuleType::Decisions => "decisions",
        RuleType::Gotchas => "gotchas",
        RuleType::Glossary => "glossary",
    }
}

fn escape_json(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
