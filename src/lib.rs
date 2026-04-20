pub mod cli;
pub mod cmd;
pub mod compile;
pub mod config;
pub mod error;
pub mod hook;
pub mod lint;
pub mod rule;
pub mod util;

use std::ffi::OsString;

use clap::Parser;

use crate::error::TenetError;

pub fn run<I, T>(args: I) -> i32
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let cli = match cli::Cli::try_parse_from(args) {
        Ok(value) => value,
        Err(err) => {
            let code = match err.kind() {
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion => 0,
                _ => 1,
            };
            let _ = err.print();
            return code;
        }
    };

    match cmd::dispatch(cli.command) {
        Ok(()) => 0,
        Err(err) => {
            eprintln!("error: {err}");
            if let Some(tenet_error) = err.downcast_ref::<TenetError>() {
                return exit_code_for_tenet_error(tenet_error);
            }
            2
        }
    }
}

fn exit_code_for_tenet_error(error: &TenetError) -> i32 {
    match error {
        TenetError::NotInGitRepo
        | TenetError::AlreadyInitialized { .. }
        | TenetError::HandwrittenConflict { .. }
        | TenetError::AbsoluteScope { .. }
        | TenetError::InvalidScope { .. }
        | TenetError::BadFrontmatterValue { .. }
        | TenetError::StaleRulesFound
        | TenetError::LintWarnings => 1,
        TenetError::LintErrors => 2,
        _ => 2,
    }
}
