//! Atomic endpoint settings outside the workload area and update binaries.

use std::fs;
use std::path::Path;

use crate::endpoint::EndpointFiles;
use crate::error::{ErrorResultExt, Result};
use crate::host_paths::CONTROL;
use crate::persistence::{atomic_private_write, private_directory};

pub(crate) const ENDPOINT_PATH: &str = "/var/lib/ciproxy/agent-url";
pub(crate) const PENDING_ENDPOINT_PATH: &str = "/var/lib/ciproxy/agent-url.pending";

pub(crate) struct LinuxEndpointFiles;

impl EndpointFiles for LinuxEndpointFiles {
    fn environment(&self) -> Result<Option<String>> {
        match std::env::var("CIPROXY_AGENT_URL") {
            Ok(value) => Ok(Some(value)),
            Err(std::env::VarError::NotPresent) => Ok(None),
            Err(error) => Err(error).context("read agent endpoint setting"),
        }
    }

    fn read(&self) -> Result<Option<String>> {
        match fs::read_to_string(ENDPOINT_PATH) {
            Ok(value) => Ok(Some(value)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error).context("read saved agent endpoint"),
        }
    }

    fn write(&self, endpoint: &str) -> Result<()> {
        private_directory(Path::new(CONTROL))?;
        atomic_private_write(Path::new(ENDPOINT_PATH), endpoint.as_bytes())
    }
}
