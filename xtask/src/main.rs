//! Command-line entry point for repository checks.

mod check;
mod cli;
mod command;
mod error;
mod file_length;
mod files;
mod markdown;

fn main() -> std::process::ExitCode {
    cli::main()
}
