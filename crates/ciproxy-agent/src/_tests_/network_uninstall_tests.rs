//! Empty uninstall reclaims reserved-name orphans before removing area or identity.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use unimock::{MockFn, Unimock, matching};

use crate::area::AreaFilesMock;
use crate::area_types::Phase;
use crate::boundaries::{ServiceMock, StoreMock};
use crate::error::Error;
use crate::fixtures::controller;
use crate::host_fixtures::inventory;
use crate::host_preparation::{FinalUninstall, PreparationMock, Uninstall, UninstallState};
use crate::network_fixtures::{Deletion, HostNetworks, commands, restored, runtime, saving};

#[tokio::test]
async fn empty_uninstall_preserves_area_and_state_until_orphan_network_cleanup_succeeds() {
    let host = Arc::new(Mutex::new(HostNetworks {
        namespaces: ["cpx1".to_owned()].into(),
        deletion: Deletion::Fail,
        ..HostNetworks::default()
    }));
    let saved = Arc::new(Mutex::new(
        br#"{"docker_budget":null,"identity":null,"mode":"Active","runners":[]}"#.to_vec(),
    ));
    let initial = restored(&saved);
    let loads = saved.clone();
    let mut current = inventory();
    current.record.as_mut().unwrap().phase = Phase::Creating;
    let removed = Arc::new(AtomicUsize::new(0));
    let area = removed.clone();
    let service = removed.clone();
    let state = removed.clone();
    let mock = Arc::new(Unimock::new((
        commands(host.clone()),
        saving(saved.clone()),
        AreaFilesMock::inspect
            .each_call(matching!())
            .answers_arc(Arc::new(move |_| Ok(current.clone()))),
        StoreMock::load
            .each_call(matching!())
            .answers_arc(Arc::new(move |_| Ok(restored(&loads)))),
        PreparationMock::remove
            .some_call(matching!())
            .answers_arc(Arc::new(move |_| {
                area.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })),
        ServiceMock::uninstall
            .some_call(matching!())
            .answers_arc(Arc::new(move |_| {
                service.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })),
        StoreMock::clear
            .some_call(matching!())
            .answers_arc(Arc::new(move |_| {
                state.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })),
    )));
    let mut agent = controller(initial, mock.as_ref().clone());
    agent.vms = Arc::new(runtime(mock.clone()));
    let uninstall = FinalUninstall {
        files: mock.clone(),
        images: mock.clone(),
        preparation: mock.clone(),
        service: mock.clone(),
        store: mock.clone(),
        agent: Arc::new(agent),
    };
    let result = uninstall.start().await;
    assert!(matches!(result, Err(Error::HostCommand { program }) if program == "ip"));
    assert_eq!(removed.load(Ordering::SeqCst), 0);
    assert_eq!(restored(&saved).pending_networks, [1].into());
    host.lock().unwrap().deletion = Deletion::Succeed;
    assert_eq!(uninstall.start().await.unwrap(), UninstallState::Complete);
    assert_eq!(restored(&saved).pending_networks, Default::default());
    assert_eq!(removed.load(Ordering::SeqCst), 3);
    assert_eq!(host.lock().unwrap().namespaces, Default::default());
}
