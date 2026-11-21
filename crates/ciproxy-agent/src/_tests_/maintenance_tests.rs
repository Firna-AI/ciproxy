//! An installed update restarts only the agent, after the hourly wait.

use std::sync::Arc;
use std::task::Poll;

use futures_util::{pin_mut, poll};
use tokio::sync::mpsc;

use unimock::{MockFn, Unimock, matching};

use crate::boundaries::{DelayMock, UpdatesMock};
use crate::error::Error;
use crate::maintenance::{HourlyMaintenance, Maintenance};
use crate::timed_fakes::ManualDelay;

#[tokio::test]
async fn update_restarts_the_agent_only_when_a_new_binary_was_installed() {
    for installed in [false, true] {
        let mock = if installed {
            Unimock::new((
                UpdatesMock::check.next_call(matching!()).returns(Ok(true)),
                UpdatesMock::restart.next_call(matching!()).returns(Ok(())),
            ))
        } else {
            Unimock::new(UpdatesMock::check.some_call(matching!()).returns(Ok(false)))
        };
        let mock = Arc::new(mock);
        HourlyMaintenance {
            updates: mock.clone(),
            delay: mock,
        }
        .cycle()
        .await
        .unwrap();
    }
}

#[tokio::test]
async fn a_failed_update_does_not_restart_the_agent_or_any_vm() {
    let mock = Arc::new(Unimock::new(
        UpdatesMock::check
            .some_call(matching!())
            .returns(Err(Error::ImageVerification)),
    ));
    assert!(matches!(
        HourlyMaintenance {
            updates: mock.clone(),
            delay: mock
        }
        .cycle()
        .await,
        Err(Error::ImageVerification)
    ));
}

#[tokio::test]
async fn hourly_run_keeps_waiting_after_a_failed_update_cycle() {
    let mock = Arc::new(Unimock::new((
        DelayMock::wait.each_call(matching!(3600)).returns(()),
        UpdatesMock::check.stub(|each| {
            each.call(matching!())
                .answers(&|_| Err(Error::ImageVerification))
                .once()
                .then()
                .answers(&|_| Ok(true))
                .once();
        }),
        UpdatesMock::restart.some_call(matching!()).returns(Ok(())),
    )));
    let (requests, mut waits) = mpsc::unbounded_channel();
    let maintenance = HourlyMaintenance {
        updates: mock.clone(),
        delay: Arc::new(ManualDelay {
            delegate: mock.clone(),
            requests,
        }),
    };
    let run = maintenance.run();
    pin_mut!(run);
    assert!(matches!(poll!(run.as_mut()), Poll::Pending));
    for _ in 0..2 {
        let wait = waits.try_recv().unwrap();
        assert_eq!(wait.seconds, 3600);
        assert_eq!(wait.release.send(()), Ok(()));
        assert!(matches!(poll!(run.as_mut()), Poll::Pending));
    }
    assert_eq!(waits.try_recv().unwrap().seconds, 3600);
}
