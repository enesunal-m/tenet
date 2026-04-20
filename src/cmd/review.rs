use std::fs;

use chrono::Local;

use crate::{rule::id::path_from_rule_id, util::paths::find_repo_root};

pub fn run(rule_id: String) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let repo_root = find_repo_root(&cwd)?;
    let path = path_from_rule_id(&repo_root, &rule_id)?;

    if !path.exists() {
        anyhow::bail!("rule not found: {rule_id}");
    }

    let content = fs::read_to_string(&path)?;
    let today = Local::now().date_naive().to_string();

    let updated = if content.starts_with("---\n") {
        upsert_reviewed_in_frontmatter(&content, &today)
    } else {
        format!("---\nreviewed: {today}\n---\n{content}")
    };

    fs::write(path, updated)?;
    Ok(())
}

fn upsert_reviewed_in_frontmatter(content: &str, today: &str) -> String {
    let remainder = &content[4..];
    if let Some(close_idx) = remainder.find("\n---\n") {
        let fm = &remainder[..close_idx];
        let body = &remainder[close_idx + "\n---\n".len()..];

        let mut lines: Vec<String> = fm
            .lines()
            .filter(|line| !line.trim_start().starts_with("reviewed:"))
            .map(String::from)
            .collect();
        lines.push(format!("reviewed: {today}"));

        format!("---\n{}\n---\n{}", lines.join("\n"), body)
    } else {
        format!("---\nreviewed: {today}\n---\n{content}")
    }
}
