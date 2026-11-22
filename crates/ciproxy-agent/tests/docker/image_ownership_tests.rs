//! One claimed file stays locked across image recovery and directory activation.

use std::sync::{Arc, mpsc};
use std::thread;
use std::time::Duration;

use ciproxy_protocol::images::ImageReport;
use unimock::{MockFn, Unimock, matching};

use crate::error::Error;
use crate::fixtures::TemporaryLayout;
use crate::image_registry::ImageRegistry;
use crate::image_space::ImageSpaceMock;
use crate::image_store::{ImageStore, LinuxImageStore};
use crate::layout::Layout;
use crate::runtime_ownership::{FileRuntimeClaims, RuntimeClaims, RuntimeWaitNoticeMock};

#[test]
fn preclaimed_runtime_initializes_images_and_stays_locked_through_recovery() {
    let temporary = TemporaryLayout::new();
    let layout = Layout {
        data: temporary.layout.data.join("area"),
        ..temporary.layout.clone()
    };
    let claims = Arc::new(FileRuntimeClaims {
        control: temporary.layout.data.clone(),
        notice: Arc::new(Unimock::new(())),
    });
    let owner = claims.claim().unwrap();
    let held = Arc::downgrade(&owner);
    assert!(!layout.data.exists());
    {
        let space = Arc::new(Unimock::new(
            ImageSpaceMock::claim
                .some_call(matching!(0))
                .answers(&|_, _| Ok(Arc::new(Unimock::new(())))),
        ));
        let store = Arc::new(LinuxImageStore::new(
            layout.clone(),
            space,
            claims.clone(),
            Some(owner),
        ));
        let registry = ImageRegistry::new(store.clone());
        let recovered = registry.recover();
        assert_eq!(recovered.as_ref().ok(), Some(&()), "{recovered:?}");
        assert_eq!(held.strong_count(), 1);
        assert!(layout.data.join("images").is_dir());
        assert_eq!(
            store.report().unwrap(),
            ImageReport {
                available: Vec::new(),
                newest: None
            }
        );
        assert!(matches!(claims.claim(), Err(Error::Internal(_))));
    }
    assert_eq!(held.strong_count(), 0);
    let next_owner = claims.claim();
    assert!(next_owner.is_ok());
}

#[test]
fn uninstall_waits_for_the_real_runtime_lock_and_notifies_once_without_a_workload_mount() {
    let temporary = TemporaryLayout::new();
    let (noticed, notice) = mpsc::channel();
    let finished = noticed.clone();
    let claims = Arc::new(FileRuntimeClaims {
        control: temporary.layout.data.clone(),
        notice: Arc::new(Unimock::new(
            RuntimeWaitNoticeMock::waiting
                .some_call(matching!())
                .answers_arc(Arc::new(move |_| {
                    noticed.send(true).unwrap();
                }))
                .once(),
        )),
    });
    let waiting = claims.clone();
    let worker = {
        let _owner = claims.claim().unwrap();
        let worker = thread::spawn(move || {
            let result = waiting.wait();
            finished.send(false).unwrap();
            result
        });
        assert!(
            notice.recv_timeout(Duration::from_secs(10)).unwrap(),
            "contention must wait instead of refusing uninstall"
        );
        assert!(!temporary.layout.data.join("area").exists());
        worker
    };
    {
        let result = worker.join().unwrap();
        assert_eq!(
            result
                .as_ref()
                .map(|_| ())
                .map_err(|error| error.to_string()),
            Ok(())
        );
        let owner = result.unwrap();
        assert!(!notice.recv_timeout(Duration::from_secs(10)).unwrap());
        assert!(matches!(claims.claim(), Err(Error::Internal(_))));
        let _lifetime = owner;
    }
    assert!(claims.claim().is_ok());
}
