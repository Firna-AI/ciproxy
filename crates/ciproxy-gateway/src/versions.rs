//! The current and preceding integer wire versions.

use ciproxy_protocol::version::ProtocolVersion;

use crate::error::{Error, ErrorResultExt, Result};

pub(crate) fn accept(version: ProtocolVersion) -> Result<()> {
    let encoded = serde_json::to_string(&ProtocolVersion::CURRENT).internal()?;
    let current: u16 = serde_json::from_str(&encoded).internal()?;
    let encoded = serde_json::to_string(&version).internal()?;
    let requested: u16 = serde_json::from_str(&encoded).internal()?;
    if requested == current || current.checked_sub(1) == Some(requested) {
        Ok(())
    } else {
        Err(Error::UnsupportedVersion)
    }
}
