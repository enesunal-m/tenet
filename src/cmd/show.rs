use std::fs;

use crate::{rule::id::path_from_rule_id, util::paths::find_repo_root};

pub fn run(rule_id: String) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let repo_root = find_repo_root(&cwd)?;
    let path = path_from_rule_id(&repo_root, &rule_id)?;

    if !path.exists() {
        anyhow::bail!("rule not found: {rule_id}");
    }

    let content = fs::read_to_string(path)?;
    print!("{content}");
    Ok(())
}
