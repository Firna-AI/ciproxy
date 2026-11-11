//! A fresh 500 GiB area finishes measured growth in one install invocation.

use std::sync::{Arc, Mutex};

use unimock::{MockFn, Unimock, matching};

use crate::area::{AreaFilesMock, AreaMountsMock, DiskArea};
use crate::area_fixtures::{area, claim, filesystem};
use crate::area_types::{AreaRecord, Inventory, Phase};
use crate::host_fixtures::record;

#[tokio::test]
async fn fresh_area_finishes_more_than_four_measured_growth_passes() {
    let starting = AreaRecord {
        source_capacity: 536_870_912_000,
        bytes: 437_653_184_512,
        overhead: 0,
        phase: Phase::Creating,
        ..record()
    };
    let saved = Arc::new(Mutex::new(Vec::new()));
    let observed = saved.clone();
    let growth = Arc::new(Mutex::new(Vec::new()));
    let sizes = growth.clone();
    let created = starting.clone();
    let mock = Arc::new(Unimock::new((
        claim(),
        AreaFilesMock::inspect
            .some_call(matching!())
            .returns(Ok(Inventory {
                record: None,
                backing: None,
                host_bytes: 536_870_912_000,
                free_bytes: 500_000_000_000,
                area_free: None,
                legacy: false,
            })),
        AreaFilesMock::create
            .some_call(matching!(536_870_912_000, 437_653_184_512, 140_000_000_000))
            .returns(Ok(created)),
        AreaMountsMock::format
            .some_call(matching!(_))
            .returns(Ok(())),
        AreaMountsMock::filesystem
            .some_call(matching!())
            .returns(Ok(filesystem())),
        AreaFilesMock::unit.some_call(matching!()).returns(Ok(())),
        AreaMountsMock::start.some_call(matching!()).returns(Ok(())),
        AreaMountsMock::mounted
            .some_call(matching!())
            .returns(Ok(true)),
        AreaMountsMock::reflinks
            .some_call(matching!())
            .returns(Ok(())),
        AreaFilesMock::directories
            .some_call(matching!())
            .returns(Ok(())),
        AreaMountsMock::available.stub(|each| {
            each.call(matching!())
                .answers(&|_| Ok(437_153_184_512))
                .once()
                .then()
                .answers(&|_| Ok(446_871_624_192))
                .once()
                .then()
                .answers(&|_| Ok(446_961_623_936))
                .once()
                .then()
                .answers(&|_| Ok(446_970_626_368))
                .once()
                .then()
                .answers(&|_| Ok(446_971_525_792))
                .once()
                .then()
                .answers(&|_| Ok(446_971_624_096));
        }),
        AreaFilesMock::grow
            .each_call(matching!(_, _))
            .answers_arc(Arc::new(move |_, record, bytes| {
                sizes.lock().unwrap().push(bytes);
                Ok(AreaRecord {
                    bytes,
                    growth_from: Some(record.bytes),
                    ..record.clone()
                })
            })),
        AreaMountsMock::grow
            .each_call(matching!())
            .answers(&|_| Ok(())),
        AreaFilesMock::record
            .each_call(matching!(_))
            .answers_arc(Arc::new(move |_, record| {
                observed.lock().unwrap().push(record.clone());
                Ok(())
            })),
    )));

    let result = area(mock.clone()).prepare().await;

    assert!(result.is_ok(), "fresh install must finish: {result:?}");
    assert_eq!(
        *growth.lock().unwrap(),
        vec![
            447_471_624_192,
            447_571_623_936,
            447_581_626_368,
            447_582_625_792,
            447_582_724_096,
        ]
    );
    assert_eq!(
        saved.lock().unwrap().last(),
        Some(&AreaRecord {
            bytes: 447_582_724_096,
            overhead: 9_929_540_096,
            phase: Phase::Ready,
            ..starting
        })
    );
}
