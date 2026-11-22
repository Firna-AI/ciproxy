//! Child commands used by the gate.

use std::path::PathBuf;
use std::process::Command;

use crate::error::{Error, IoResultExt, Result};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CommandSpec {
    pub(crate) program: String,
    pub(crate) args: Vec<String>,
    /// Environment variables whose values are paths from the repository root.
    pub(crate) root_paths: Vec<(String, String)>,
}

impl CommandSpec {
    pub(crate) fn new(program: &str, args: &[&str]) -> Self {
        Self {
            program: program.to_owned(),
            args: args.iter().map(|arg| (*arg).to_owned()).collect(),
            root_paths: Vec::new(),
        }
    }

    /// Set `name` to the absolute form of `path`, a path from the root.
    pub(crate) fn root_path(mut self, name: &str, path: &str) -> Self {
        self.root_paths.push((name.to_owned(), path.to_owned()));
        self
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
        let mut child = Command::new(&command.program);
        for (name, path) in &command.root_paths {
            child.env(name, self.root.join(path));
        }
        let status = child
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
