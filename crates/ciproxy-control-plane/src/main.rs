//! Composition root of the hosted ciproxy HTTP service.

mod cli;

fn main() -> std::process::ExitCode {
    cli::main()
}
