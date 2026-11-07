//! Optional hosted metering and billing composition without changing ApiState.

use std::sync::Arc;

use axum::{Extension, Router};
use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_billing::service::api::BillingApi;
use ciproxy_billing::service::paid_review::PaidReview;
use ciproxy_metering::report::catalog::PriceList;
use ciproxy_metering::service::Metering;
use ciproxy_overflow::api::Overflow;

use crate::router::router;
use crate::state::ApiState;

/// Trusted page collaborators supplied by the primary hosted composition root.
pub struct PageServices {
    /// Existing Stripe billing service, configured with operator-owned secrets.
    pub billing: Arc<dyn BillingApi>,
    /// Existing store-backed metering service.
    pub metering: Arc<dyn Metering>,
    /// Dated Blacksmith and GitHub USD catalogs, absent until configured.
    pub price_lists: Option<[PriceList; 2]>,
    /// Authoritative Stripe preview for paid changes, absent until composed.
    pub paid_review: Option<Arc<dyn PaidReview>>,
    /// Existing overflow service for live queue refusal and spend observations.
    pub overflow: Option<Arc<dyn Overflow>>,
    /// The exact trusted sizing shared with OverflowConfig for the Boat image.
    pub boat_root_sizing: Option<RootSizing>,
}

/// Build the dashboard router with billing and metering. Mount under `/api`.
pub fn router_with_pages(state: ApiState, services: Arc<PageServices>) -> Router {
    router(state).layer(Extension(services))
}

/// Mount the raw Stripe webhook at the dashboard host's `/hooks/stripe`.
pub fn stripe_router(state: ApiState, billing: Arc<dyn BillingApi>) -> Router {
    crate::handlers::stripe::router(state, billing)
}
