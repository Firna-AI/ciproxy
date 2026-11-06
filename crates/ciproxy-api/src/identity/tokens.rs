//! Explicit credential serialization only at the encryption boundary.

use chrono::{DateTime, Utc};
use secrecy::{ExposeSecret, SecretString};
use serde::Serialize;

use crate::error::{ErrorResultExt, Result};

use super::Grant;

#[derive(Serialize)]
struct EncodedGrant<'a> {
    access_token: &'a str,
    expires_at: Option<DateTime<Utc>>,
    refresh_token: Option<&'a str>,
    refresh_expires_at: Option<DateTime<Utc>>,
}

pub(super) fn encode(grant: &Grant) -> Result<SecretString> {
    Ok(serde_json::to_string(&EncodedGrant {
        access_token: grant.access_token.expose_secret(),
        expires_at: grant.expires_at,
        refresh_token: grant
            .refresh_token
            .as_ref()
            .map(ExposeSecret::expose_secret),
        refresh_expires_at: grant.refresh_expires_at,
    })
    .internal()?
    .into())
}
