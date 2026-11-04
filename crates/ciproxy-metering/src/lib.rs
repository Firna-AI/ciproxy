//! Pure workspace metering and savings calculations over trusted store facts.

#![warn(unreachable_pub)]

pub mod period;
pub mod pricing;
pub mod report;
pub mod service;
pub mod usage;

#[cfg(test)]
#[path = "_tests_/support.rs"]
mod test_support;
