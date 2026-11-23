//! Sealed pending setup and one-use OAuth bindings.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Utc};
use ciproxy_store_interface::dashboard::records::LoginState;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
use uuid::Uuid;

use crate::error::{Error, ErrorResultExt, Result};
use crate::state::query;

use super::authentication::digest;
use super::pending::BrowserState;
use super::setup::SetupTarget;
use super::{AccessService, LoginAttempt, OAUTH_BINDING_LIFETIME};

#[derive(Deserialize, Serialize)]
#[serde(tag = "purpose", rename_all = "snake_case")]
pub(super) enum Purpose {
    SignIn,
    Setup {
        installation: Option<SetupTarget>,
        install_state: Option<String>,
        request_only: bool,
    },
}

#[derive(Deserialize, Serialize)]
pub(super) struct Binding {
    pub(super) id: Uuid,
    pub(super) nonce: String,
    pub(super) purpose: Purpose,
    pub(super) previous: Option<BrowserState>,
}

#[derive(Deserialize, Serialize)]
struct PendingSetup {
    purpose: Purpose,
    expires_at: DateTime<Utc>,
}

impl AccessService {
    pub(super) async fn seal_pending_setup(&self, purpose: Purpose) -> Result<SecretString> {
        let id = self.runtime.identifier();
        let pending = PendingSetup {
            purpose,
            expires_at: self.runtime.now() + OAUTH_BINDING_LIFETIME,
        };
        let sealed = self
            .cipher
            .seal(id, serde_json::to_string(&pending).internal()?.into())
            .await?;
        binding_cookie("setup", id, &sealed)
    }

    pub(super) async fn open_pending_setup(&self, cookie: &str) -> Option<Purpose> {
        let mut pieces = cookie.splitn(3, '.');
        if pieces.next()? != "setup" {
            return None;
        }
        let id = pieces.next()?.parse().ok()?;
        let sealed = URL_SAFE_NO_PAD.decode(pieces.next()?).ok()?;
        let opened = self.cipher.open(id, sealed).await.ok()?;
        let pending: PendingSetup = serde_json::from_str(opened.expose_secret()).ok()?;
        (self.runtime.now() < pending.expires_at).then_some(pending.purpose)
    }

    pub(super) async fn start(
        &self,
        purpose: Purpose,
        previous: Option<&str>,
    ) -> Result<LoginAttempt> {
        let state = self.runtime.secret()?;
        let verifier = self.runtime.secret()?;
        let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.expose_secret().as_bytes()));
        let id = self.runtime.identifier();
        let prior = if let Some(cookie) = previous {
            self.open_browser(cookie).await.ok()
        } else {
            None
        };
        let binding = Binding {
            id,
            nonce: state.expose_secret().to_owned(),
            purpose,
            previous: prior,
        };
        let sealed = self
            .cipher
            .seal(id, serde_json::to_string(&binding).internal()?.into())
            .await?;
        let binding = binding_cookie("oauth", id, &sealed)?;
        let saved = LoginState {
            id,
            digest: digest(state.expose_secret()),
            verifier: self.cipher.seal(id, verifier).await?,
            expires_at: self.runtime.now() + OAUTH_BINDING_LIFETIME,
        };
        query(self.store.clone(), move |store| {
            store.save_login_state(&saved)
        })
        .await?;
        Ok(LoginAttempt {
            url: self
                .identity
                .authorization_url(state.expose_secret(), &challenge),
            binding,
        })
    }

    pub(super) async fn consume_binding(&self, state: &str, cookie: &str) -> Result<Binding> {
        let mut pieces = cookie.splitn(3, '.');
        if pieces.next() != Some("oauth") {
            return Err(Error::Unauthorized);
        }
        let Some(id) = pieces.next().and_then(|value| Uuid::parse_str(value).ok()) else {
            return Err(Error::Unauthorized);
        };
        let Some(value) = pieces.next() else {
            return Err(Error::Unauthorized);
        };
        let bytes = match URL_SAFE_NO_PAD.decode(value) {
            Ok(value) => value,
            Err(_) => return Err(Error::Unauthorized),
        };
        let opened = self.cipher.open(id, bytes).await?;
        let binding: Binding = match serde_json::from_str(opened.expose_secret()) {
            Ok(value) => value,
            Err(_) => return Err(Error::Unauthorized),
        };
        if binding.id != id || !bool::from(digest(state).ct_eq(&digest(&binding.nonce))) {
            return Err(Error::Unauthorized);
        }
        Ok(binding)
    }
}

fn binding_cookie(kind: &str, id: Uuid, sealed: &[u8]) -> Result<SecretString> {
    let value = format!("{kind}.{id}.{}", URL_SAFE_NO_PAD.encode(sealed));
    if value.len() > 3800 {
        return Err(Error::Unavailable);
    }
    Ok(value.into())
}
