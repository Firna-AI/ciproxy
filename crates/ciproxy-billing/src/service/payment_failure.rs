//! Preserve reserved value through rejected and ambiguous priced mutations.

use ciproxy_store_interface::billing::commands::BillingCommand;
use ciproxy_store_interface::billing::payments::Settlement;
use ciproxy_store_interface::billing::store::BillingSession;

use crate::error::{Error, Result, ResultExt};

use super::payment_work::money_held;
use super::runtime::BillingService;

impl BillingService {
    pub(super) fn payment_failure(
        &self,
        session: &mut dyn BillingSession,
        command: &BillingCommand,
        error: &Error,
    ) -> Result<()> {
        let mut work = session.payment(command.id).internal()?;
        if let (Some(work), Error::StripeValidationRejected { proof }) = (&mut work, error) {
            work.rejection = Some(*proof);
            work.settlement = Settlement::Restoring;
            self.save_payment(session, work)?;
            self.restore_payment(session, work)?;
        } else if work.as_ref().is_some_and(money_held) {
            return Ok(());
        }
        self.release_rejected(session, error)?;
        if let Some(work) = &mut work {
            self.save_payment(session, work)?;
        }
        Ok(())
    }
}
