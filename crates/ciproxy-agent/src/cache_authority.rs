//! Host-owned original endpoint origins and rights survive agent restarts.

use ciproxy_protocol::runtime::{CachePermissions, RuntimeCredentials};
use reqwest::Url;
use serde::{Deserialize, Serialize};

/// Durable authority for this VM's one assignment, without runtime secrets.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) enum CacheAuthority {
    Waiting,
    GitHubOnly,
    Sealed(CacheSeal),
}

/// Original origins and rights, fixed before the first running-state save.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct CacheSeal {
    source_gate: SourceGate,
    permissions: CachePermissions,
    cache_origin: EndpointOrigin,
    results_origin: EndpointOrigin,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
enum SourceGate {
    HostDecision,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
enum EndpointOrigin {
    Absent,
    Http {
        scheme: String,
        host: String,
        port: u16,
    },
}

/// Runtime values checked against this host's durable original authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedRuntime {
    runtime: RuntimeCredentials,
}

impl CheckedRuntime {
    pub(crate) fn into_runtime(self) -> RuntimeCredentials {
        self.runtime
    }
}

impl CacheAuthority {
    pub(crate) fn handoff(
        &mut self,
        first: bool,
        initial_authority: bool,
        mut runtime: RuntimeCredentials,
    ) -> Option<CheckedRuntime> {
        if matches!(self, Self::Waiting) {
            *self = match (first, initial_authority, CacheSeal::from_runtime(&runtime)) {
                (true, true, Some(seal)) => Self::Sealed(seal),
                _ => Self::GitHubOnly,
            };
        }
        let Self::Sealed(seal) = self else {
            return None;
        };
        if EndpointOrigin::parse(runtime.cache_url.expose_secret())? != seal.cache_origin
            || EndpointOrigin::parse(runtime.results_url.expose_secret())? != seal.results_origin
        {
            return None;
        }
        runtime.permissions = seal.permissions;
        Some(CheckedRuntime { runtime })
    }
}

impl CacheSeal {
    fn from_runtime(runtime: &RuntimeCredentials) -> Option<Self> {
        Some(Self {
            source_gate: SourceGate::HostDecision,
            permissions: runtime.permissions,
            cache_origin: EndpointOrigin::parse(runtime.cache_url.expose_secret())?,
            results_origin: EndpointOrigin::parse(runtime.results_url.expose_secret())?,
        })
    }
}

impl EndpointOrigin {
    fn parse(value: &str) -> Option<Self> {
        if value.is_empty() {
            return Some(Self::Absent);
        }
        let url = Url::parse(value).ok()?;
        if !matches!(url.scheme(), "http" | "https") {
            return None;
        }
        Some(Self::Http {
            scheme: url.scheme().to_owned(),
            host: url.host_str()?.to_owned(),
            port: url.port_or_known_default()?,
        })
    }
}
