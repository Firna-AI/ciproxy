//! Command environment and legacy endpoint storage for retained identities.

use std::fs;

use crate::endpoint::EndpointFiles;
use crate::error::{ErrorResultExt, Result};

pub(crate) const ENDPOINT_PATH: &str = "/var/lib/ciproxy/agent-url";
pub(crate) const PENDING_ENDPOINT_PATH: &str = "/var/lib/ciproxy/agent-url.pending";

pub(crate) struct ControlEndpointFiles;

impl EndpointFiles for ControlEndpointFiles {
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

    fn remove(&self) -> Result<()> {
        match fs::remove_file(ENDPOINT_PATH) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error).context("remove legacy agent endpoint"),
        }
    }
}
