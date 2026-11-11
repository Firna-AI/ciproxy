//! Typed pending items and preview-line identity for payment-side transfers.

use serde::{Deserialize, Serialize};

/// One Stripe invoice item, including an attachment acquired during a renewal.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InvoiceItem {
    /// Exact provider item identifier.
    pub id: String,
    /// Signed integer cents before invoice-level adjustments.
    pub amount: i64,
    /// Provider currency, checked against the USD review.
    pub currency: String,
    /// Owning Stripe customer.
    pub customer: String,
    /// Attached invoice, absent while pending.
    pub invoice: Option<String>,
    /// Subscription which produced this item, when scoped.
    pub parent: Option<ItemParent>,
}

/// Invoice-item source scope.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ItemParent {
    /// A generated subscription adjustment.
    pub subscription_details: Option<ItemSubscription>,
}

/// Owned subscription identity for a generated pending adjustment.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ItemSubscription {
    /// Provider subscription identifier.
    pub subscription: String,
}

impl InvoiceItem {
    /// Read the optional subscription scope without losing the customer owner.
    pub fn subscription_id(&self) -> Option<&str> {
        self.parent
            .as_ref()?
            .subscription_details
            .as_ref()
            .map(|details| details.subscription.as_str())
    }
}

#[derive(Clone, Deserialize)]
pub(crate) struct PreviewLine {
    pub(crate) id: String,
    pub(crate) amount: i64,
    pub(crate) currency: String,
    pub(crate) parent: Option<LineParent>,
    #[serde(default)]
    pub(crate) discount_amounts: Vec<LineDiscount>,
}

#[derive(Clone, Deserialize)]
pub(crate) struct LineDiscount {
    pub(crate) amount: i64,
}

#[derive(Clone, Deserialize)]
pub(crate) struct LineParent {
    pub(crate) invoice_item_details: Option<LineSource>,
    pub(crate) subscription_item_details: Option<LineSource>,
}

#[derive(Clone, Deserialize)]
pub(crate) struct LineSource {
    pub(crate) invoice_item: Option<String>,
}

impl PreviewLine {
    pub(crate) fn pending_id(&self) -> Option<&str> {
        let parent = self.parent.as_ref()?;
        parent
            .invoice_item_details
            .as_ref()
            .and_then(|item| item.invoice_item.as_deref())
            .or_else(|| {
                parent
                    .subscription_item_details
                    .as_ref()
                    .and_then(|item| item.invoice_item.as_deref())
            })
    }
}
