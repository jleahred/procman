use std::env;

use clap::{Parser, Subcommand};
#[derive(Parser)]
#[command(name = "procman")]
#[command(about = env!("CARGO_PKG_DESCRIPTION"), version, long_about=None)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    /// Execute with the given processes configuration file
    Run {
        /// configuration file to use. toml format with info about the processes to run
        processes_filename: String,
    },
    /// check the given processes configuration file
    Check {
        /// configuration file to use. toml format with info about the processes to run
        processes_filename: String,
    },
    /// Generate a UID to be used in the config file
    Tui {
        processes_filename: String,
    },
    Uid,
    /// Expand config templates and show on stdout
    ExpandTemplates {
        /// configuration file to use. toml format with info about the processes to run
        processes_filename: String,
    },
}

pub(crate) fn parse() -> Cli {
    Cli::parse()
}
