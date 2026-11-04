//! Clap argument parsing and composition of repository check boundaries.

mod args;

use std::path::Path;
use std::process::ExitCode;

use clap::Parser;

use crate::check::runner::{CheckRunner, WorkspaceCheck};
use crate::command::ProcessRunner;
use crate::files::DiskRepository;

use self::args::{Cli, Commands};

pub(crate) fn main() -> ExitCode {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let commands = ProcessRunner { root: root.clone() };
    let repository = DiskRepository { root };
    let check: &dyn CheckRunner = &WorkspaceCheck {
        commands: &commands,
        repository: &repository,
    };
    let result = match Cli::parse().command {
        Commands::Check(args) => check.run(&args.include, &args.exclude, args.keep_going),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
