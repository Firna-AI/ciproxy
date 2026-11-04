//! An installed update restarts only the agent, after the hourly wait.

use std::sync::Arc;

use unimock::{MockFn, Unimock, matching};

use crate::boundaries::UpdatesMock;
use crate::error::Error;
use crate::maintenance::{HourlyMaintenance, Maintenance};

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
