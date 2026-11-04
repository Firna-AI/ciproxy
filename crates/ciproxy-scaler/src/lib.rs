//! Pure runner placement and retirement decisions for one workspace.

pub mod actions;
pub mod boat;
pub mod decision;
pub mod snapshot;

mod capacity;
mod lifecycle;
mod overflow;
mod placement;
mod size_policy;
mod state;
mod warm;

#[cfg(test)]
#[path = "_tests_/mod.rs"]
mod tests;
