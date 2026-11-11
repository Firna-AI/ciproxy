//! One checked control-plane endpoint for install and daemon startup.

use std::str::FromStr;
use std::sync::Arc;

use reqwest::Url;

use crate::error::{Error, Result};

const DEFAULT: &str = "wss://app.ciproxy.dev/agent/";

#[derive(Clone, Debug, Eq, PartialEq)]
/// A checked secure WebSocket endpoint for one control plane.
pub struct AgentEndpoint(String);

impl AgentEndpoint {
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for AgentEndpoint {
    fn default() -> Self {
        Self(DEFAULT.to_owned())
    }
}

impl FromStr for AgentEndpoint {
    type Err = Error;

    fn from_str(value: &str) -> Result<Self> {
        match Url::parse(value) {
            Ok(url) if url.scheme() == "wss" && url.host_str().is_some() => {
                Ok(Self(url.to_string()))
            }
            _ => Err(Error::AgentEndpoint),
        }
    }
}

#[cfg_attr(test, unimock::unimock(api = EndpointFilesMock))]
pub(crate) trait EndpointFiles: Send + Sync {
    fn environment(&self) -> Result<Option<String>>;
    fn read(&self) -> Result<Option<String>>;
    fn write(&self, endpoint: &str) -> Result<()>;
}

#[cfg_attr(test, unimock::unimock(api = EndpointSettingsMock))]
pub(crate) trait EndpointSettings: Send + Sync {
    fn resolve(&self, supplied: Option<&AgentEndpoint>) -> Result<AgentEndpoint>;
    fn save(&self, endpoint: &AgentEndpoint) -> Result<()>;
}

pub(crate) struct SavedEndpoint {
    pub(crate) files: Arc<dyn EndpointFiles>,
}

impl EndpointSettings for SavedEndpoint {
    fn resolve(&self, supplied: Option<&AgentEndpoint>) -> Result<AgentEndpoint> {
        if let Some(endpoint) = supplied {
            return Ok(endpoint.clone());
        }
        let value = match self.files.read()? {
            Some(value) => Some(value),
            None => self.files.environment()?,
        };
        match value {
            Some(value) => value.parse(),
            None => Ok(AgentEndpoint::default()),
        }
    }

    fn save(&self, endpoint: &AgentEndpoint) -> Result<()> {
        self.files.write(endpoint.as_str())
    }
}

#[cfg(test)]
#[path = "_tests_/endpoint_tests.rs"]
mod endpoint_tests;
