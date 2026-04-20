pub mod add;
pub mod compile;
pub mod edit;
pub mod init;
pub mod lint;
pub mod list;
pub mod migrate;
pub mod review;
pub mod show;
pub mod stale;

use crate::cli::Commands;

pub fn dispatch(command: Commands) -> anyhow::Result<()> {
    match command {
        Commands::Init { force, no_hook } => init::run(force, no_hook),
        Commands::Add {
            rule_type,
            scope,
            owner,
            priority,
            title,
        } => add::run(rule_type, scope, owner, priority, title),
        Commands::List {
            rule_type,
            scope,
            owner,
            stale,
            json,
        } => list::run(rule_type, scope, owner, stale, json),
        Commands::Show { rule_id } => show::run(rule_id),
        Commands::Edit { rule_id } => edit::run(rule_id),
        Commands::Review { rule_id } => review::run(rule_id),
        Commands::Stale { grace, owner, json } => stale::run(grace, owner, json),
        Commands::Compile {
            dry_run,
            exclude_stale,
        } => compile::run(dry_run, exclude_stale),
        Commands::Lint {
            check_compiled,
            check_secrets,
            quiet,
        } => lint::run(check_compiled, check_secrets, quiet),
        Commands::Migrate {
            from_path,
            yes,
            mapping,
        } => migrate::run(from_path, yes, mapping),
        Commands::Version => {
            println!("tenet {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
    }
}
