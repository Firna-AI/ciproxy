//! Shared dashboard composition for hosted routes and durable billing recovery.

use std::sync::Arc;

use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_api::pages::PageServices;
use ciproxy_billing::alerts::BillingAlerts;
use ciproxy_billing::clock::Clock;
use ciproxy_billing::service::runtime::{BillingService, Redirects};
use ciproxy_billing::stripe::http::client::HttpStripe;
use ciproxy_billing::webhook::SigningSecret;
use ciproxy_metering::report::catalog::PriceList;
use ciproxy_metering::service::Metering;
use ciproxy_overflow::api::Overflow;
use ciproxy_store_interface::billing::store::BillingStore;
use secrecy::{ExposeSecret, SecretString};

use crate::billing::{BillingCancellations, BillingMaintenance};
use crate::error::{ErrorResultExt, Result};

/// Private Stripe inputs, supplied by the process configuration.
pub struct StripeSettings {
    /// Trusted Stripe API endpoint, replaceable by an HTTP peer in tests.
    pub endpoint: String,
    /// Operator-owned Stripe credential, never sent to dashboard clients.
    pub secret_key: SecretString,
    /// Signing secret for the raw host-root webhook.
    pub signing_secret: SigningSecret,
    /// Trusted dashboard return locations.
    pub redirects: Redirects,
}

/// Trusted page inputs from the deployed configuration and overflow owner.
pub struct PageConfig {
    /// Dated vendor lists used by the savings report.
    pub price_lists: [PriceList; 2],
    /// The exact value moved into the configured overflow service.
    pub boat_root_sizing: Option<RootSizing>,
}

/// The process shares billing and metering between pages and runtime workers.
pub struct PageComposition {
    /// Page collaborators mounted by the production HTTP root.
    pub services: Arc<PageServices>,
    /// Independent retry path for last-organization billing requests.
    pub cancellations: Arc<dyn BillingMaintenance>,
}

impl PageComposition {
    /// Construct the blocking Stripe client and billing service on a worker.
    pub async fn billing(
        store: Arc<dyn BillingStore>,
        clock: Arc<dyn Clock>,
        alerts: Arc<dyn BillingAlerts>,
        settings: StripeSettings,
    ) -> Result<Arc<BillingService>> {
        tokio::task::spawn_blocking(move || {
            let stripe = HttpStripe::with_base_url(
                settings.secret_key.expose_secret().to_owned(),
                &settings.endpoint,
            )
            .internal()?;
            Ok(Arc::new(BillingService::new(
                store,
                Arc::new(stripe),
                clock,
                alerts,
                settings.signing_secret,
                settings.redirects,
            )))
        })
        .await
        .internal()?
    }

    /// Share one billing instance across pages, paid review, hook, and recovery.
    pub fn new(
        store: Arc<dyn BillingStore>,
        billing: Arc<BillingService>,
        metering: Arc<dyn Metering>,
        overflow: Option<Arc<dyn Overflow>>,
        config: PageConfig,
    ) -> Self {
        Self {
            cancellations: Arc::new(BillingCancellations {
                store,
                billing: billing.clone(),
            }),
            services: Arc::new(PageServices {
                billing: billing.clone(),
                paid_review: Some(billing),
                metering,
                overflow,
                price_lists: Some(config.price_lists),
                boat_root_sizing: config.boat_root_sizing,
            }),
        }
    }
}
