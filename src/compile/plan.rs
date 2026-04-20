use std::{collections::BTreeMap, path::PathBuf};

use chrono::NaiveDate;

use crate::rule::{self, Rule};

use super::render::render_agents_md;

/// Planned write operation for a generated `AGENTS.md` file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedWrite {
    pub path: PathBuf,
    pub content: String,
}

#[derive(Debug, Clone, Copy)]
pub struct PlanOptions {
    pub include_stale: bool,
    pub segregate_stale: bool,
    pub grace_days: i64,
    pub today: NaiveDate,
}

pub fn build_plan(
    repo_root: &std::path::Path,
    rules: &[Rule],
    options: PlanOptions,
) -> Vec<PlannedWrite> {
    let mut buckets: BTreeMap<PathBuf, Vec<Rule>> = BTreeMap::new();

    for rule in rules.iter().cloned() {
        let stale = is_stale(&rule, options.grace_days, options.today);
        if stale && !options.include_stale {
            continue;
        }

        let scope = rule.frontmatter.scope.as_deref().unwrap_or("**");
        let anchor = rule::scope::compute_anchor(repo_root, scope).anchor;
        buckets.entry(anchor).or_default().push(rule);
    }

    if buckets.is_empty() {
        let content = render_agents_md(repo_root, PathBuf::new(), &[], &[], options.today);
        return vec![PlannedWrite {
            path: repo_root.join("AGENTS.md"),
            content,
        }];
    }

    let all_anchors: Vec<PathBuf> = buckets.keys().cloned().collect();

    buckets
        .into_iter()
        .map(|(anchor, mut rules)| {
            rules.sort_by(|left, right| {
                priority_rank(left.frontmatter.priority)
                    .cmp(&priority_rank(right.frontmatter.priority))
                    .then_with(|| left.id.cmp(&right.id))
            });

            let (fresh, stale): (Vec<Rule>, Vec<Rule>) = if options.segregate_stale {
                rules
                    .into_iter()
                    .partition(|rule| !is_stale(rule, options.grace_days, options.today))
            } else {
                (rules, Vec::new())
            };

            PlannedWrite {
                path: repo_root.join(&anchor).join("AGENTS.md"),
                content: render_agents_md(repo_root, anchor, &fresh, &stale, options.today)
                    .replace("{{SEE_ALSO}}", &render_see_also(&all_anchors)),
            }
        })
        .collect()
}

fn render_see_also(anchors: &[PathBuf]) -> String {
    let mut paths: Vec<String> = anchors
        .iter()
        .filter(|a| !a.as_os_str().is_empty())
        .map(|a| format!("{}/AGENTS.md", a.display()))
        .collect();
    paths.sort();
    if paths.is_empty() {
        return String::new();
    }
    let mut out = String::from("## See also\n\n");
    for p in paths {
        out.push_str(&format!("- `{p}`\n"));
    }
    out.push('\n');
    out
}

fn is_stale(rule: &Rule, grace_days: i64, today: NaiveDate) -> bool {
    rule.frontmatter
        .reviewed
        .map(|reviewed| reviewed + chrono::Days::new(grace_days.max(0) as u64) < today)
        .unwrap_or(false)
}

fn priority_rank(priority: crate::rule::frontmatter::Priority) -> u8 {
    match priority {
        crate::rule::frontmatter::Priority::High => 0,
        crate::rule::frontmatter::Priority::Normal => 1,
        crate::rule::frontmatter::Priority::Low => 2,
    }
}
