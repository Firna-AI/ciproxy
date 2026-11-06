//! The single workspace banner shared by every dashboard page.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use super::billing::PlanOffer;
use super::links::ActivationView;

/// The payer action, replaced with a request for an authorized person.
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum BannerAction {
    /// Open the restricted Stripe portal.
    UpdateCard,
    /// Open Plans with the saved workspace context.
    ChoosePlan,
    /// The current role cannot pay.
    AskAdminOrBilling,
}

/// One inactive organization and every reason it cannot start jobs.
#[derive(Serialize, ToSchema)]
pub struct InactiveOrganization {
    /// Stable organization identity.
    pub id: String,
    /// Organization login.
    pub login: String,
    /// Role-aware activation reasons and actions.
    pub activation: ActivationView,
}

/// One banner after payment, inactive-organization, and soft-limit priority.
#[derive(Serialize, ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PageBanner {
    /// Paid coverage stays active while Stripe retries.
    PaymentFailed {
        /// Last failed payment date.
        failed_at: Option<DateTime<Utc>>,
        /// Next Stripe retry date when available.
        next_retry_at: Option<DateTime<Utc>>,
        /// Role-aware recovery action.
        action: BannerAction,
    },
    /// Paid coverage ended but all own machines continue to run jobs.
    SubscriptionEnded {
        /// Exclusive end of this notice.
        until: DateTime<Utc>,
        /// Role-aware plan action.
        action: BannerAction,
    },
    /// Other organizations keep running while these wait.
    InactiveOrganizations {
        /// Inactive organizations and all their hold reasons.
        organizations: Vec<InactiveOrganization>,
    },
    /// Jobs continue with no extra charge above advisory limits.
    SoftLimit {
        /// Cheapest paid choice that covers forecast use.
        offer: PlanOffer,
        /// Role-aware plan action.
        action: BannerAction,
    },
}

/// Shared banner response for existing workspace pages.
#[derive(Serialize, ToSchema)]
pub struct PageBannerView {
    /// At most one banner, absent when no rule requires it.
    pub banner: Option<PageBanner>,
}

/// Select the Organizations tab's local activation notice when applicable.
#[derive(Default, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct PageBannerQuery {
    /// Organizations uses its inactive notice before a soft-limit notice.
    #[param(inline)]
    pub page: Option<BannerPage>,
}

/// Page-specific banner scope.
#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum BannerPage {
    /// The Organizations tab of Settings.
    Organizations,
}
