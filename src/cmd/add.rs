use std::{
    fs,
    io::{self, IsTerminal, Read},
    process::Command,
};

use dialoguer::Input;

use crate::{
    cli::{Priority, RuleType},
    rule::frontmatter::parse_rule_document,
    util::paths::find_repo_root,
};

pub fn run(
    rule_type: RuleType,
    scope: Option<String>,
    owner: Option<String>,
    priority: Option<Priority>,
    title: Option<String>,
) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let repo_root = find_repo_root(&cwd)?;

    let interactive = io::stdout().is_terminal();
    let scope = resolve_field(scope, interactive, "Scope", "**")?;
    let owner = resolve_optional_field(owner, interactive, "Owner")?;
    let priority = resolve_priority(priority, interactive)?;
    let title = resolve_field(title, interactive, "Title", "new-rule")?;

    if !interactive && (scope.is_empty() || owner.is_none() || title.is_empty()) {
        anyhow::bail!("non-interactive mode requires --scope, --owner, --priority, --title");
    }

    let mut body = String::new();
    if !io::stdin().is_terminal() {
        io::stdin().read_to_string(&mut body)?;
    }

    let slug = slugify(&title);
    let dir = repo_root.join(".context").join(rule_type_dir(&rule_type));
    fs::create_dir_all(&dir)?;

    let path = next_available_path(&dir, &slug);
    fs::write(
        &path,
        render_new_rule(&scope, owner.as_deref(), &priority, &body),
    )?;

    open_editor_with_validation(&path)?;

    let id = crate::rule::id::rule_id_from_path(&path)?;
    println!("{id}");

    Ok(())
}

fn resolve_field(
    current: Option<String>,
    interactive: bool,
    prompt: &str,
    default_value: &str,
) -> anyhow::Result<String> {
    if let Some(value) = current {
        return Ok(value);
    }

    if !interactive {
        anyhow::bail!("missing required flag in non-interactive mode");
    }

    Input::new()
        .with_prompt(prompt)
        .default(default_value.to_string())
        .interact_text()
        .map_err(Into::into)
}

fn resolve_optional_field(
    current: Option<String>,
    interactive: bool,
    prompt: &str,
) -> anyhow::Result<Option<String>> {
    if let Some(value) = current {
        return Ok(Some(value));
    }

    if !interactive {
        return Ok(None);
    }

    let value: String = Input::new()
        .with_prompt(prompt)
        .allow_empty(true)
        .interact_text()?;
    if value.is_empty() {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}

fn resolve_priority(current: Option<Priority>, interactive: bool) -> anyhow::Result<Priority> {
    if let Some(value) = current {
        return Ok(value);
    }

    if !interactive {
        anyhow::bail!("missing required --priority in non-interactive mode");
    }

    let value: String = Input::new()
        .with_prompt("Priority (high|normal|low)")
        .default("normal".to_string())
        .interact_text()?;

    match value.as_str() {
        "high" => Ok(Priority::High),
        "normal" => Ok(Priority::Normal),
        "low" => Ok(Priority::Low),
        _ => anyhow::bail!("invalid priority"),
    }
}

fn render_new_rule(scope: &str, owner: Option<&str>, priority: &Priority, body: &str) -> String {
    let owner_line = owner
        .map(|value| format!("owner: {value}\n"))
        .unwrap_or_default();

    format!(
        "---\nscope: \"{scope}\"\n{owner_line}priority: {}\n---\n{}",
        priority_value(priority),
        body
    )
}

fn open_editor_with_validation(path: &std::path::Path) -> anyhow::Result<()> {
    for _ in 0..3 {
        open_editor(path)?;
        let content = fs::read_to_string(path)?;
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

fn slugify(title: &str) -> String {
    let mut out = String::new();
    for ch in title.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if (ch.is_ascii_whitespace() || ch == '-') && !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}

fn next_available_path(dir: &std::path::Path, slug: &str) -> std::path::PathBuf {
    let mut index = 1;
    loop {
        let name = if index == 1 {
            format!("{slug}.md")
        } else {
            format!("{slug}-{index}.md")
        };
        let path = dir.join(name);
        if !path.exists() {
            return path;
        }
        index += 1;
    }
}

fn priority_value(priority: &Priority) -> &'static str {
    match priority {
        Priority::High => "high",
        Priority::Normal => "normal",
        Priority::Low => "low",
    }
}

fn rule_type_dir(rule_type: &RuleType) -> &'static str {
    match rule_type {
        RuleType::Invariants => "invariants",
        RuleType::Conventions => "conventions",
        RuleType::Decisions => "decisions",
        RuleType::Gotchas => "gotchas",
        RuleType::Glossary => "glossary",
    }
}
