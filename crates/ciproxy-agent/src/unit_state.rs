//! The state of the process that owns a VM, with no other dependency, so
//! the regression targets can include it by path.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Unit status used to preserve live VMs and reclaim exited VMs.
pub enum UnitState {
    /// The unit process is live or starting.
    Active,
    /// The unit process exited with this code.
    Exited(i32),
    /// No loaded unit has this locally generated name.
    Absent,
}
