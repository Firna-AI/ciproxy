//! Coherent stored inputs for one workspace savings report.

use crate::metering::facts::MachineRegistration;
use crate::metering::history::CostHistory;
use crate::metering::totals::DailyTotal;

/// Registrations, range totals, and monthly costs from one database snapshot.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SavingsSnapshot {
    /// Current and removed own-machine registrations for historical cost dates.
    pub machines: Vec<MachineRegistration>,
    /// Anonymous daily totals in the requested half-open UTC date range.
    pub days: Vec<DailyTotal>,
    /// Monthly machine and plan prices, with the trusted workspace start time.
    pub costs: CostHistory,
}
