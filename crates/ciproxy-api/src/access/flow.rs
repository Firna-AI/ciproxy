//! One-use OAuth binding with authenticated purpose and prior sign-in metadata.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::Duration;
use ciproxy_store_interface::dashboard::records::LoginState;
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
use uuid::Uuid;

use crate::error::{Error, ErrorResultExt, Result};
use crate::state::query;

use super::authentication::digest;
use super::pending::BrowserState;
use super::setup::SetupTarget;
use super::{AccessService, LoginAttempt};

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

impl AccessService {
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
        let value = format!("oauth.{id}.{}", URL_SAFE_NO_PAD.encode(sealed));
        if value.len() > 3800 {
            return Err(Error::Unavailable);
        }
        let saved = LoginState {
            id,
            digest: digest(state.expose_secret()),
            verifier: self.cipher.seal(id, verifier).await?,
            expires_at: self.runtime.now() + Duration::minutes(10),
        };
        query(self.store.clone(), move |store| {
            store.save_login_state(&saved)
        })
        .await?;
        Ok(LoginAttempt {
            url: self
                .identity
                .authorization_url(state.expose_secret(), &challenge),
            binding: value.into(),
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
