use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "tenet")]
#[command(about = "Compile .context rules into AGENTS.md", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Init {
        #[arg(long)]
        force: bool,
        #[arg(long = "no-hook")]
        no_hook: bool,
    },
    Add {
        rule_type: RuleType,
        #[arg(long)]
        scope: Option<String>,
        #[arg(long)]
        owner: Option<String>,
        #[arg(long)]
        priority: Option<Priority>,
        #[arg(long)]
        title: Option<String>,
    },
    List {
        #[arg(long = "type")]
        rule_type: Option<RuleType>,
        #[arg(long)]
        scope: Option<String>,
        #[arg(long)]
        owner: Option<String>,
        #[arg(long)]
        stale: bool,
        #[arg(long)]
        json: bool,
    },
    Show {
        rule_id: String,
    },
    Edit {
        rule_id: String,
    },
    Review {
        rule_id: String,
    },
    Stale {
        #[arg(long)]
        grace: Option<u32>,
        #[arg(long)]
        owner: Option<String>,
        #[arg(long)]
        json: bool,
    },
    Compile {
        #[arg(long = "dry-run")]
        dry_run: bool,
        #[arg(long = "exclude-stale")]
        exclude_stale: bool,
    },
    Lint {
        #[arg(long = "check-compiled")]
        check_compiled: bool,
        #[arg(long = "check-secrets")]
        check_secrets: bool,
        #[arg(long)]
        quiet: bool,
    },
    Migrate {
        #[arg(long = "from")]
        from_path: String,
        #[arg(long)]
        yes: bool,
        #[arg(long)]
        mapping: Option<String>,
    },
    Version,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum RuleType {
    Invariants,
    Conventions,
    Decisions,
    Gotchas,
    Glossary,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum Priority {
    High,
    Normal,
    Low,
}
