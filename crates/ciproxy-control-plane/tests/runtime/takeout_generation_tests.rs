//! A confirming installation refresh keeps Busy proof after a failed check.

use ciproxy_control_plane::error::Error;
use ciproxy_control_plane::runtime::OwnMachines;

use super::overflow_support::Harness;
use super::overflow_takeout_support::prepare;
use super::settled::Settled;
use super::takeout_installations::Probe;
use super::takeout_support::{Answer, Call, Remote};

#[tokio::test]
async fn confirming_takeout_lookup_preserves_recovered_busy_generation() {
    let mut harness = Harness::new(false, false).await;
    harness.queued();
    harness.queued();
    harness.runtime.settled_tick().await.unwrap();
    let remote = Remote::new(Answer::Busy);
    remote
        .answers
        .lock()
        .unwrap()
        .insert(101, Answer::DeleteUnavailable);
    let event = prepare(&mut harness, remote.clone());
    let probe = Probe::present();
    harness.runtime.installations = probe.clone();
    harness.runtime.overflow = None;
    let pending = harness.runtime.lifecycle(event.clone()).await;
    assert!(matches!(pending, Err(Error::StateChanged)), "{pending:?}");
    let failed = harness.runtime.quiesce().await;
    assert!(matches!(failed, Err(Error::StateChanged)), "{failed:?}");
    remote.answers.lock().unwrap().insert(101, Answer::Busy);
    let pending = harness.runtime.lifecycle(event.clone()).await;
    assert!(matches!(pending, Err(Error::StateChanged)), "{pending:?}");
    harness.runtime.quiesce().await.unwrap();
    assert_eq!(*probe.calls.lock().unwrap(), vec![43, 43]);
    let accepted = harness.runtime.lifecycle(event.clone()).await;
    assert!(accepted.is_ok(), "{accepted:?}");
    harness.runtime.quiesce_lifecycle(&event).await.unwrap();
    let calls = remote.calls.lock().unwrap();
    assert_eq!(
        calls
            .iter()
            .filter(|call| **call == Call::Remove(100))
            .count(),
        2
    );
    assert_eq!(
        calls
            .iter()
            .filter(|call| **call == Call::Remove(101))
            .count(),
        2
    );
    assert_eq!(calls.len(), 4);
}
