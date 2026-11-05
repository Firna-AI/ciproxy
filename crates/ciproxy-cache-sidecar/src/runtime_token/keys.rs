//! Shared public keys with periodic and unknown-key refresh cooldowns.

use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_protocol::jobs::UnknownJobScopeReason;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::jwk::JwkSet;
use tokio::sync::Mutex;

use crate::clock::Clock;

use super::source::KeySource;

const REFRESH_SECONDS: u64 = 300;
const UNKNOWN_KEY_COOLDOWN: u64 = 60;

/// Public-key selection and refresh at the machine boundary.
#[async_trait]
pub trait SigningKeys: Send + Sync {
    /// Select one public key, with one rate-limited unknown-key refresh.
    async fn key(&self, kid: &str) -> Result<DecodingKey, UnknownJobScopeReason>;
    /// Refresh at most every five minutes, including failed attempts.
    async fn refresh(&self);
}

#[derive(Default)]
struct State {
    keys: Option<JwkSet>,
    regular_attempt: Option<u64>,
    unknown_attempt: Option<u64>,
}

/// One key cache shared by every session on a machine.
pub struct CachedKeys {
    source: Arc<dyn KeySource>,
    clock: Arc<dyn Clock>,
    state: Mutex<State>,
}

impl CachedKeys {
    /// Compose public-key IO and time at the machine's composition root.
    pub fn new(source: Arc<dyn KeySource>, clock: Arc<dyn Clock>) -> Self {
        Self {
            source,
            clock,
            state: Mutex::new(State::default()),
        }
    }

    async fn regular(&self, state: &mut State) {
        let now = self.clock.now();
        if state
            .regular_attempt
            .is_none_or(|last| now.saturating_sub(last) >= REFRESH_SECONDS)
        {
            state.regular_attempt = Some(now);
            if !self.fetch(state).await {
                state.keys = None;
            }
        }
    }

    async fn fetch(&self, state: &mut State) -> bool {
        match self.source.fetch().await {
            Ok(keys) => {
                state.keys = Some(keys);
                true
            }
            Err(_) => {
                tracing::warn!("Runtime-token public keys unavailable");
                false
            }
        }
    }
}

fn select(state: &State, kid: &str) -> Result<DecodingKey, UnknownJobScopeReason> {
    let Some(keys) = &state.keys else {
        return Err(UnknownJobScopeReason::KeysUnavailable);
    };
    let Some(key) = keys.find(kid) else {
        return Err(UnknownJobScopeReason::UnknownKey);
    };
    match DecodingKey::from_jwk(key) {
        Ok(key) => Ok(key),
        Err(_) => Err(UnknownJobScopeReason::InvalidSignature),
    }
}

#[async_trait]
impl SigningKeys for CachedKeys {
    async fn key(&self, kid: &str) -> Result<DecodingKey, UnknownJobScopeReason> {
        let mut state = self.state.lock().await;
        self.regular(&mut state).await;
        match select(&state, kid) {
            Err(UnknownJobScopeReason::UnknownKey) => {
                let now = self.clock.now();
                if state
                    .unknown_attempt
                    .is_none_or(|last| now.saturating_sub(last) >= UNKNOWN_KEY_COOLDOWN)
                {
                    state.unknown_attempt = Some(now);
                    self.fetch(&mut state).await;
                }
                select(&state, kid)
            }
            result => result,
        }
    }

    async fn refresh(&self) {
        let mut state = self.state.lock().await;
        self.regular(&mut state).await;
    }
}
