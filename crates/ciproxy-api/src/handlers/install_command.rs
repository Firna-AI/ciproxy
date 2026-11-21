//! Environment-bound install arguments from immutable operator configuration.

use ciproxy_protocol::secret::Secret;
use reqwest::Url;

use crate::error::{ErrorResultExt, Result};
use crate::state::ApiConfig;

pub(super) fn command(config: &ApiConfig, token: &Secret) -> Result<String> {
    let origin = Url::parse(&config.public_origin).internal()?;
    let endpoint = format!("wss://{}/agent/", origin.authority());
    let arguments = if endpoint == "wss://app.ciproxy.dev/agent/" {
        String::new()
    } else {
        format!(" --agent-url '{}'", endpoint.replace('\'', "'\\''"))
    };
    Ok(format!(
        "curl -fsSL {} | sudo sh -s -- install{arguments} --token {}",
        config.download_url,
        token.expose_secret()
    ))
}
