//! Literal runtime capacity and root-sizing answers for API boundary tests.

use std::sync::Arc;

use ciproxy_agent::root_sizing::RootFit;
use ciproxy_api::capacity::{Capacity, MachineCapacity, MachineCapacityMock};
use ciproxy_protocol::resources::Slots;
use unimock::{MockFn, Unimock, matching};

pub(super) fn standard() -> Arc<dyn MachineCapacity> {
    Arc::new(Unimock::new((
        MachineCapacityMock::capacity
            .each_call(matching!(_))
            .returns(Capacity {
                slots: Slots(6),
                job_disk_bytes: Some(180_000_000_000),
                image_disk_bytes: Some(58_300_000_000),
            }),
        MachineCapacityMock::required_root
            .each_call(matching!((_, _)))
            .returns(RootFit::Holds(80_000_000_000)),
    )))
}

pub(super) fn unavailable() -> Arc<dyn MachineCapacity> {
    Arc::new(Unimock::new((
        MachineCapacityMock::capacity
            .each_call(matching!(_))
            .returns(Capacity {
                slots: Slots(6),
                job_disk_bytes: Some(180_000_000_000),
                image_disk_bytes: None,
            }),
        MachineCapacityMock::required_root
            .each_call(matching!((_, _)))
            .returns(RootFit::Unknown),
    )))
}

pub(super) fn mixed() -> Arc<dyn MachineCapacity> {
    Arc::new(Unimock::new((
        MachineCapacityMock::capacity.each_call(matching!((snapshot) if snapshot.machine.hostname=="builder")).returns(Capacity {
            slots:Slots(6),job_disk_bytes:Some(180_000_000_000),image_disk_bytes:Some(40_000_000_000),
        }),
        MachineCapacityMock::capacity.each_call(matching!((snapshot) if snapshot.machine.hostname=="builder-new")).returns(Capacity {
            slots:Slots(6),job_disk_bytes:Some(180_000_000_000),image_disk_bytes:Some(90_000_000_000),
        }),
        MachineCapacityMock::required_root.each_call(matching!((snapshot,free) if snapshot.machine.hostname=="builder"&&*free==20_000_000_000)).returns(RootFit::Holds(80_000_000_000)),
        MachineCapacityMock::required_root.each_call(matching!((snapshot,free) if snapshot.machine.hostname=="builder-new"&&*free==20_000_000_000)).returns(RootFit::Holds(120_000_000_000)),
        MachineCapacityMock::required_root.each_call(matching!((snapshot,free) if snapshot.machine.hostname=="builder"&&*free==100_000_000_000)).returns(RootFit::Holds(180_000_000_000)),
        MachineCapacityMock::required_root.each_call(matching!((snapshot,free) if snapshot.machine.hostname=="builder-new"&&*free==100_000_000_000)).returns(RootFit::Holds(180_000_000_000)),
    )))
}

/// A label that asks for more free disk than the machine's image has.
pub(super) fn above_image() -> Arc<dyn MachineCapacity> {
    Arc::new(Unimock::new((
        MachineCapacityMock::capacity
            .each_call(matching!(_))
            .returns(Capacity {
                slots: Slots(6),
                job_disk_bytes: Some(180_000_000_000),
                image_disk_bytes: Some(100_000_000_000),
            }),
        MachineCapacityMock::required_root
            .each_call(matching!((_, _)))
            .returns(RootFit::AboveImage),
    )))
}
