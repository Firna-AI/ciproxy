//! Child commands used by the gate.

use std::path::PathBuf;
use std::process::Command;

use crate::error::{Error, IoResultExt, Result};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CommandSpec {
    pub(crate) program: String,
    pub(crate) args: Vec<String>,
}

impl CommandSpec {
    pub(crate) fn new(program: &str, args: &[&str]) -> Self {
        Self {
            program: program.to_owned(),
            args: args.iter().map(|arg| (*arg).to_owned()).collect(),
        }
    }
}

#[cfg_attr(test, unimock::unimock(api = [CommandRunMock]))]
pub(crate) trait CommandRunner {
    fn run(&self, command: &CommandSpec) -> Result<()>;
}

pub(crate) struct ProcessRunner {
    pub(crate) root: PathBuf,
}

impl CommandRunner for ProcessRunner {
    fn run(&self, command: &CommandSpec) -> Result<()> {
        eprintln!("$ {} {}", command.program, command.args.join(" "));
        let status = Command::new(&command.program)
            .args(&command.args)
            .current_dir(&self.root)
            .status()
            .internal()?;
        if status.success() {
            Ok(())
        } else {
            Err(Error::CommandFailed {
                program: command.program.clone(),
                status,
            })
        }
    }
}
