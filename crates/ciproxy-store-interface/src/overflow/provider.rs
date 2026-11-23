//! Stable identifiers and fixed facts for supported overflow providers.

use ciproxy_protocol::platform::Platform;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// The provider that owns an overflow account, sandbox, or cost.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OverflowProvider {
    /// Boat, also the provider of records saved before provider identity.
    #[default]
    Boat,
}

impl OverflowProvider {
    /// The stored text identifier.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Boat => "boat",
        }
    }

    /// The fixed facts associated with the stored provider identifier.
    pub const fn descriptor(self) -> &'static ProviderDescriptor {
        match self {
            Self::Boat => &BOAT,
        }
    }
}

/// Fixed provider facts independent of the deployment and customer account.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProviderDescriptor {
    /// Provider brand name.
    pub display_name: &'static str,
    /// An example key prefix, for recognition only.
    pub key_hint: &'static str,
    /// Path to the provider's connection guide on the ciproxy site.
    pub guide_path: &'static str,
    /// Lifetime limit for each new sandbox.
    pub lifetime: LifetimeRule,
    /// Billing account choice shown during connection, if the provider has one.
    pub billing_account_kind: Option<BillingAccountKind>,
    /// Disk layout that the provider offers for jobs.
    pub disk_model: DiskModel,
    /// How the deployed image reaches the sandbox.
    pub image_source_kind: ImageSourceKind,
    /// Platforms on which the provider runs ciproxy jobs.
    pub platforms: &'static [Platform],
    /// Whether admission needs a published start-fee bound.
    pub start_fee: StartFeeRule,
}

/// One-shot lifetime limits in seconds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LifetimeRule {
    /// Normal lifetime for a sandbox.
    pub standard_seconds: u32,
    /// Shorter lifetime when a provider reports a trial, if applicable.
    pub trial_seconds: Option<u32>,
}

impl LifetimeRule {
    /// Select the lifetime for the account's current trial status.
    pub const fn seconds(self, trial: bool) -> u32 {
        match (trial, self.trial_seconds) {
            (true, Some(seconds)) => seconds,
            _ => self.standard_seconds,
        }
    }
}

/// The kind of billing account a customer can choose.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BillingAccountKind {
    /// Personal or organization wallet.
    Wallet,
}

/// How the provider supplies job disk space.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiskModel {
    /// The sandbox root holds the image and all job disk space.
    RootOnly,
}

/// How a provider restores the software for a job.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImageSourceKind {
    /// A shared saved-snapshot name or share code.
    SharedSnapshot,
}

/// The start-fee facts a provider needs for budget admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StartFeeRule {
    /// The provider charges no start fee.
    NoFee,
    /// A start requires a published bound, including a published zero.
    RequiredBound,
}

impl StartFeeRule {
    /// Return the safe fee bound, or None when the provider needs to supply one.
    pub const fn admission_bound(self, published: Option<Decimal>) -> Option<Decimal> {
        match self {
            Self::NoFee => Some(Decimal::ZERO),
            Self::RequiredBound => published,
        }
    }
}

const BOAT: ProviderDescriptor = ProviderDescriptor {
    display_name: "Boat",
    key_hint: "boat_live_",
    guide_path: "/docs/guides/connect-boat/",
    lifetime: LifetimeRule {
        standard_seconds: 21600,
        trial_seconds: Some(7200),
    },
    billing_account_kind: Some(BillingAccountKind::Wallet),
    disk_model: DiskModel::RootOnly,
    image_source_kind: ImageSourceKind::SharedSnapshot,
    platforms: &[Platform::LinuxX64],
    start_fee: StartFeeRule::RequiredBound,
};

#[cfg(test)]
#[path = "_tests_/provider_tests.rs"]
mod provider_tests;
