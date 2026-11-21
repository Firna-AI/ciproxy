//! Safe hosted operator events without provider bodies or payment credentials.

use unimock::unimock;
use uuid::Uuid;

/// Credential-free diagnostic fields for a paid change.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BillingAlert {
    /// Workspace which owns the operation.
    pub workspace_id: Uuid,
    /// Immutable review identifier.
    pub quote_id: Uuid,
    /// Durable payment command identifier.
    pub command_id: Uuid,
    /// Exact invoice identifier when it is known.
    pub invoice_id: Option<String>,
    /// Reviewed integer US cents.
    pub expected_cents: u64,
    /// Actual integer US cents, present for a mismatch.
    pub actual_cents: Option<u64>,
    /// Closed diagnostic category with no provider text.
    pub reason: AlertReason,
}

/// Operator event categories which never decide entitlement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AlertReason {
    /// A paid invoice differs from its authoritative review.
    InvoiceMismatch,
    /// Financial or invoice evidence remains unavailable for recovery.
    RecoveryPending,
}

/// Operator event boundary, independent of email or pager delivery.
#[unimock(api = BillingAlertsMock)]
pub trait BillingAlerts: Send + Sync {
    /// Raise a safe event after durable recovery state exists.
    fn emit(&self, alert: &BillingAlert);
}

/// Emit through the operator logging subscriber installed by the hosted process.
pub struct HostedBillingAlerts;

impl BillingAlerts for HostedBillingAlerts {
    fn emit(&self, alert: &BillingAlert) {
        tracing::error!(
            target: "ciproxy_billing::operator",
            code = ?alert.reason,
            workspace_id = %alert.workspace_id,
            quote_id = %alert.quote_id,
            command_id = %alert.command_id,
            invoice_id = ?alert.invoice_id,
            expected_cents = alert.expected_cents,
            actual_cents = ?alert.actual_cents,
            currency = "usd",
            "paid billing operation needs operator attention"
        );
    }
}
