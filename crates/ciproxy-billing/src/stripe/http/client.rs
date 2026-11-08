//! Blocking Stripe HTTP adapter with redacted credentials and fixed timeouts.

use std::fmt;
use std::time::Duration;

use reqwest::blocking::{Client, Response};
use reqwest::redirect::Policy;
use url::Url;

use super::rejection::validation;
use crate::error::{Error, Result, ResultExt};

/// Stripe API version shared by the HTTP client and webhook endpoint.
pub const API_VERSION: &str = "2026-10-28.endive";

/// Blocking HTTP provider, constructed only at runtime composition.
pub struct HttpStripe {
    client: Client,
    base_url: Url,
    secret_key: String,
}

impl fmt::Debug for HttpStripe {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HttpStripe")
            .field("secret_key", &"[redacted]")
            .finish_non_exhaustive()
    }
}

impl HttpStripe {
    /// Construct a provider for the official Stripe endpoint.
    pub fn new(secret_key: String) -> Result<Self> {
        Self::with_base_url(secret_key, "https://api.stripe.com/")
    }

    /// Construct a provider for a fake HTTP endpoint in integration tests.
    pub fn with_base_url(secret_key: String, base_url: &str) -> Result<Self> {
        Ok(Self {
            client: Client::builder()
                .timeout(Duration::from_secs(15))
                .redirect(Policy::none())
                .build()
                .internal()?,
            base_url: Url::parse(base_url).internal()?,
            secret_key,
        })
    }

    pub(super) fn get(&self, path: &str, query: &[(String, String)]) -> Result<Response> {
        let mut url = self.base_url.join(path).internal()?;
        url.query_pairs_mut().extend_pairs(query);
        let request = self
            .client
            .get(url)
            .bearer_auth(&self.secret_key)
            .header("Stripe-Version", API_VERSION)
            .send();
        response(request, false)
    }

    pub(super) fn post(
        &self,
        path: &str,
        form: &[(String, String)],
        key: Option<&str>,
    ) -> Result<Response> {
        let url = self.base_url.join(path).internal()?;
        let mut request = self
            .client
            .post(url)
            .bearer_auth(&self.secret_key)
            .header("Stripe-Version", API_VERSION)
            .form(form);
        if let Some(key) = key {
            request = request.header("Idempotency-Key", key);
        }
        response(request.send(), true)
    }

    pub(super) fn delete(
        &self,
        path: &str,
        form: &[(String, String)],
        key: Option<&str>,
    ) -> Result<Response> {
        let url = self.base_url.join(path).internal()?;
        let mut request = self
            .client
            .delete(url)
            .bearer_auth(&self.secret_key)
            .header("Stripe-Version", API_VERSION)
            .form(form);
        if let Some(key) = key {
            request = request.header("Idempotency-Key", key);
        }
        response(request.send(), true)
    }
}

fn response(
    result: std::result::Result<Response, reqwest::Error>,
    mutation: bool,
) -> Result<Response> {
    let response = match result {
        Ok(response) => response,
        Err(_) => return Err(Error::StripeUnavailable),
    };
    let status = response.status().as_u16();
    if mutation && response.status().is_client_error() && status != 409 && status != 429 {
        if let Some(proof) = validation(response, status) {
            return Err(Error::StripeValidationRejected { proof });
        }
        return Err(Error::StripeRejected { status });
    }
    if !response.status().is_success() {
        return Err(Error::StripeHttp {
            status: response.status().as_u16(),
        });
    }
    Ok(response)
}
