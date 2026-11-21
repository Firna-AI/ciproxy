//! Owned host preparation fixtures. No host resources are read.

use crate::area_types::{AreaRecord, Backing, Inventory, Phase};
use crate::host_release::Release;

pub(crate) fn release() -> Release {
    Release {
        archive: "559c0d956a06cfcc6425c4146eadd9f80de07c33386ccd010852ac410f80f885",
        firecracker: "8f0efd545168f15ee78363b5b0ef79178a09179bf73a234833082ebc88f45da4",
        jailer: "6a92940281f6157aefabfdf7f5ae0969c5ed931811c8a4052f22325b2547ed30",
    }
}

pub(crate) fn archive() -> Vec<u8> {
    vec![
        31, 139, 8, 0, 0, 0, 0, 0, 2, 255, 237, 211, 77, 14, 130, 48, 16, 64, 225, 30, 133, 11,
        128, 211, 202, 143, 183, 49, 13, 41, 9, 202, 106, 0, 195, 241, 45, 186, 208, 160, 11, 77,
        12, 137, 225, 125, 155, 105, 218, 197, 44, 154, 167, 161, 11, 190, 15, 233, 197, 102, 182,
        202, 36, 157, 14, 229, 177, 204, 119, 77, 171, 161, 86, 95, 159, 131, 46, 158, 204, 247,
        36, 170, 68, 110, 83, 94, 167, 136, 219, 63, 206, 243, 189, 45, 92, 92, 148, 136, 89, 193,
        216, 15, 94, 227, 122, 179, 77, 77, 59, 13, 163, 134, 228, 233, 195, 13, 182, 67, 223, 247,
        127, 242, 109, 247, 139, 244, 63, 235, 223, 150, 139, 254, 115, 87, 208, 255, 170, 253,
        223, 63, 156, 30, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 254, 205, 21, 186, 244, 121, 9, 0,
        40, 0, 0,
    ]
}

pub(crate) fn record() -> AreaRecord {
    AreaRecord {
        version: 2,
        uuid: "c7081f45-8824-42d8-bfb3-7c65d56c1bc5".to_owned(),
        source_capacity: 250_000_000_000,
        bytes: 226_000_003_072,
        device: 3,
        inode: 92,
        phase: Phase::Ready,
        image_budget: 140_000_000_000,
        overhead: 3_500_003_072,
        growth_from: None,
    }
}

pub(crate) fn inventory() -> Inventory {
    Inventory {
        record: Some(record()),
        backing: Some(Backing {
            device: 3,
            inode: 92,
            bytes: 226_000_003_072,
            safe: true,
        }),
        host_bytes: 250_000_000_000,
        free_bytes: 65_000_000_000,
        area_free: Some(50_000_000_000),
        legacy: false,
    }
}
