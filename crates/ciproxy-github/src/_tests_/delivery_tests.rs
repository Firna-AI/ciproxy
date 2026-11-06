//! Consumer failure and a lost confirmation both retain ordered lifecycle delivery.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use ciproxy_store_interface::github_lifecycle::{LifecycleDelivery, LifecycleEvent, RemovalCause};
use unimock::{MockFn, Unimock, matching};

use crate::app::{GitHubApp, LifecycleOutputMock};
use crate::error::Error;
use crate::persistence::lifecycle::LifecycleStoreMock;

use super::coordination::lease;
use super::support::{app, organization};

#[tokio::test]
async fn lifecycle_delivery_confirms_only_after_acceptance_and_retries_unconfirmed_acceptance() {
    let accepted = Arc::new(AtomicUsize::new(0));
    let attempts = Arc::new(AtomicUsize::new(0));
    let confirmations = Arc::new(AtomicUsize::new(0));
    let observed = accepted.clone();
    let observed_attempts = attempts.clone();
    let observed_confirmations = confirmations.clone();
    let event = LifecycleEvent::Removed {
        organization: organization(),
        last_organization: true,
        cause: RemovalCause::Owner,
    };
    let service = app(Unimock::new((
        lease(),
        LifecycleStoreMock::organizations
            .each_call(matching!())
            .answers(&|_| Ok(vec![17])),
        LifecycleStoreMock::pending
            .each_call(matching!(17))
            .answers_arc(Arc::new(move |_, _| {
                Ok(vec![LifecycleDelivery {
                    sequence: 41,
                    event: event.clone(),
                }])
            })),
        LifecycleOutputMock::changed
            .each_call(matching!(_))
            .answers_arc(Arc::new(move |_, _| {
                if observed_attempts.fetch_add(1, Ordering::SeqCst) == 0 {
                    return Err(Error::AccessUnavailable(503));
                }
                observed.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })),
        LifecycleStoreMock::confirm
            .each_call(matching!(17, 41))
            .answers_arc(Arc::new(move |_, _, _| {
                if observed_confirmations.fetch_add(1, Ordering::SeqCst) == 0 {
                    return Err(Error::AccessUnavailable(504));
                }
                Ok(())
            })),
    )));
    assert_eq!(
        service.pending_lifecycle_organizations().await.unwrap(),
        vec![17]
    );
    assert!(matches!(
        service.replay_lifecycle(17).await,
        Err(Error::AccessUnavailable(503))
    ));
    assert_eq!(confirmations.load(Ordering::SeqCst), 0);
    assert!(matches!(
        service.replay_lifecycle(17).await,
        Err(Error::AccessUnavailable(504))
    ));
    assert_eq!(accepted.load(Ordering::SeqCst), 1);
    assert!(matches!(service.replay_lifecycle(17).await, Ok(())));
    assert_eq!(confirmations.load(Ordering::SeqCst), 2);
    assert_eq!(attempts.load(Ordering::SeqCst), 3);
    assert_eq!(accepted.load(Ordering::SeqCst), 2);
}
