//! One claimed file stays locked across image recovery and directory activation.

use std::sync::Arc;

use ciproxy_protocol::images::ImageReport;
use unimock::{MockFn, Unimock, matching};

use crate::error::Error;
use crate::fixtures::TemporaryLayout;
use crate::image_registry::ImageRegistry;
use crate::image_space::ImageSpaceMock;
use crate::image_store::{ImageStore, LinuxImageStore};
use crate::layout::Layout;
use crate::runtime_ownership::{LinuxRuntimeClaims, RuntimeClaims};

#[test]
fn preclaimed_runtime_initializes_images_and_stays_locked_through_recovery() {
    let temporary = TemporaryLayout::new();
    let layout = Layout {
        data: temporary.layout.data.join("area"),
        ..temporary.layout.clone()
    };
    let claims = Arc::new(LinuxRuntimeClaims {
        control: temporary.layout.data.clone(),
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
