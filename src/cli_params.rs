use std::{env, path::PathBuf};

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
        processes_filename: PathBuf,
    },
    /// check the given processes configuration file
    Check {
        /// configuration file to use. toml format with info about the processes to run
        processes_filename: PathBuf,
    },
    /// Terminal User Interface watch and to manage processes
    Tui,
    /// Generate a UID to be used in the config file
    Uid,
    /// Expand config templates and show on stdout
    ExpandTemplates {
        /// configuration file to use. toml format with info about the processes to run
        processes_filename: PathBuf,
    },
    /// Generate a monitor process file with minimum example with the given filename
    GenFile { filename: Option<PathBuf> },
}

pub(crate) fn parse() -> Cli {
    Cli::parse()
}
