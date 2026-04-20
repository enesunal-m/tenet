use std::{collections::BTreeMap, fs, path::Path};

use dialoguer::{Input, Select};

use crate::util::paths::find_repo_root;

pub fn run(from_path: String, yes: bool, mapping: Option<String>) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let repo_root = find_repo_root(&cwd)?;

    let source_path = repo_root.join(from_path);
    let source = fs::read_to_string(&source_path)?;
    let sections = parse_sections(&source);

    let mapping_table = if yes {
        let mapping_path = mapping.ok_or_else(|| anyhow::anyhow!("--yes requires --mapping"))?;
        load_mapping(&repo_root.join(mapping_path))?
    } else {
        BTreeMap::new()
    };

    for section in sections {
        if section.body.trim().is_empty() {
            continue;
        }

        let (rule_type, scope) = if yes {
            let mapped = mapping_table.get(&section.title).ok_or_else(|| {
                anyhow::anyhow!("missing mapping for section '{}'", section.title)
            })?;
            (mapped.rule_type.clone(), mapped.scope.clone())
        } else {
            let options = [
                "invariants",
                "conventions",
                "decisions",
                "gotchas",
                "glossary",
                "skip",
            ];
            let selection = Select::new()
                .with_prompt(format!("Type for section '{}'?", section.title))
                .items(options)
                .default(0)
                .interact()?;
            let selected = options[selection].to_string();
            if selected == "skip" {
                continue;
            }
            let scope: String = Input::new()
                .with_prompt("Scope")
                .default("**".to_string())
                .interact_text()?;
            (selected, scope)
        };

        let slug = slugify(&section.title);
        let target = repo_root
            .join(".context")
            .join(&rule_type)
            .join(format!("{slug}.md"));

        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = format!("---\nscope: \"{}\"\n---\n{}", scope, section.body.trim());
        fs::write(&target, content)?;
        println!("wrote {}", target.display());
    }

    println!(
        "migration complete; source left untouched: {}",
        source_path.display()
    );
    Ok(())
}

#[derive(Debug, Clone)]
struct MappingItem {
    rule_type: String,
    scope: String,
}

fn load_mapping(path: &Path) -> anyhow::Result<BTreeMap<String, MappingItem>> {
    let raw = fs::read_to_string(path)?;
    let value: toml::Value = toml::from_str(&raw)?;
    let table = value
        .get("sections")
        .and_then(|v| v.as_table())
        .ok_or_else(|| anyhow::anyhow!("mapping file must have [sections] table"))?;

    let mut out = BTreeMap::new();
    for (title, entry) in table {
        let rule_type = entry
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("mapping entry missing type: {title}"))?;
        let scope = entry
            .get("scope")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("mapping entry missing scope: {title}"))?;

        out.insert(
            title.clone(),
            MappingItem {
                rule_type: rule_type.to_string(),
                scope: scope.to_string(),
            },
        );
    }

    Ok(out)
}

#[derive(Debug, Clone)]
struct Section {
    title: String,
    body: String,
}

fn parse_sections(markdown: &str) -> Vec<Section> {
    let mut sections = Vec::new();
    let mut current_title: Option<String> = None;
    let mut current_body = String::new();

    for line in markdown.lines() {
        if line.starts_with("# ") || line.starts_with("## ") {
            if let Some(title) = current_title.take() {
                sections.push(Section {
                    title,
                    body: current_body.trim().to_string(),
                });
                current_body.clear();
            }
            current_title = Some(line.trim_start_matches('#').trim().to_string());
            continue;
        }

        if current_title.is_some() {
            current_body.push_str(line);
            current_body.push('\n');
        }
    }

    if let Some(title) = current_title {
        sections.push(Section {
            title,
            body: current_body.trim().to_string(),
        });
    }

    sections
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
