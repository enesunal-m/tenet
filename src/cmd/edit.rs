use std::{fs, process::Command};

use crate::{
    rule::{frontmatter::parse_rule_document, id::path_from_rule_id},
    util::paths::find_repo_root,
};

pub fn run(rule_id: String) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let repo_root = find_repo_root(&cwd)?;
    let path = path_from_rule_id(&repo_root, &rule_id)?;

    if !path.exists() {
        anyhow::bail!("rule not found: {rule_id}");
    }

    for _ in 0..3 {
        open_editor(&path)?;
        let content = fs::read_to_string(&path)?;
        if parse_rule_document(&content).is_ok() {
            return Ok(());
        }
        eprintln!("error: invalid frontmatter, reopening editor");
    }

    anyhow::bail!("validation failed after 3 attempts")
}

fn open_editor(path: &std::path::Path) -> anyhow::Result<()> {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| String::from("vi"));
    let status = Command::new(editor).arg(path).status()?;
    if !status.success() {
        anyhow::bail!("editor exited with failure")
    }
    Ok(())
}
