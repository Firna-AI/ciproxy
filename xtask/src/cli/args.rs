//! Public command line and phase filters.

use clap::{Args, Parser, Subcommand};

use crate::check::phases::Phase;

#[derive(Parser)]
#[command(about = "Repository verification", version)]
pub(super) struct Cli {
    #[command(subcommand)]
    pub(super) command: Commands,
}

#[derive(Subcommand)]
pub(super) enum Commands {
    /// Run the gate in phase order. Stop at the first failed phase by default.
    Check(CheckArgs),
}

#[derive(Args)]
pub(super) struct CheckArgs {
    /// Run every selected phase and summarize failures at the end.
    #[arg(long)]
    pub(super) keep_going: bool,
    /// Run only these phases. Repeat or separate names with commas.
    #[arg(long, value_enum, value_delimiter = ',')]
    pub(super) include: Vec<Phase>,
    /// Skip these phases. Exclusion takes priority over inclusion.
    #[arg(long, value_enum, value_delimiter = ',')]
    pub(super) exclude: Vec<Phase>,
}
