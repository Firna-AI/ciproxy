//! Definitive refusals retain their exact retry, plan, billing, and trial effects.

use chrono::Duration;
use ciproxy_overflow::api::{Overflow, StartOutcome, WaitReason};
use ciproxy_overflow::provider::BoatAccountApiMock;
use sandbox_interface::{
    BillingRefusalReason, PlanSizeRefusalReason, SandboxLifecycleMock, SandboxLifetime,
    SandboxStartRefusal, SandboxStartRetry, TemporaryStartRefusalReason,
};
use unimock::{MockFn, Unimock, matching};

use super::harness::{Harness, Runtime};

use super::fixtures::{at, capabilities, facts, id, request};

#[tokio::test]
async fn refusals_persist_exact_retry_plan_billing_and_trial_state() {
    let cases = [
        SandboxStartRefusal::Temporary {
            reason: TemporaryStartRefusalReason::RateLimited,
            retry: SandboxStartRetry {
                retry_after: Some(std::time::Duration::from_secs(60)),
                next_start_at: Some(at() + Duration::seconds(90)),
            },
        },
        SandboxStartRefusal::PlanSize {
            reason: PlanSizeRefusalReason::PlanRequired,
        },
        SandboxStartRefusal::Billing {
            reason: BillingRefusalReason::BillingRequired,
        },
        SandboxStartRefusal::TrialAutoStopRequired,
    ];
    for refusal in cases {
        let response = refusal.clone();
        let backend = capabilities(Unimock::new((
            BoatAccountApiMock::account
                .some_call(matching!())
                .answers(&|_| Ok(facts())),
            SandboxLifecycleMock::create_sandbox
                .some_call(matching!(_))
                .answers_arc(std::sync::Arc::new(move |_, _| {
                    Err(sandbox_interface::Error::SandboxStartRefused {
                        backend_id: "boat".to_owned(),
                        refusal: response.clone(),
                    })
                })),
        )));
        let harness = Harness::new(backend, (), Runtime::Start);
        assert_eq!(
            harness.service.start(id(1), request()).await.unwrap(),
            StartOutcome::Waiting(WaitReason::Refused(refusal.clone()))
        );
        let snapshot = harness.snapshot();
        let account = &snapshot.accounts[0].state;
        let expected_retry = matches!(refusal, SandboxStartRefusal::Temporary { .. })
            .then_some(at() + Duration::seconds(90));
        assert_eq!(account.retry_at, expected_retry, "{refusal:?}");
        assert_eq!(
            account.refused_sizes,
            if matches!(refusal, SandboxStartRefusal::PlanSize { .. }) {
                vec!["large"]
            } else {
                Vec::<&str>::new()
            },
            "{refusal:?}"
        );
        assert_eq!(
            account.billing_required,
            matches!(refusal, SandboxStartRefusal::Billing { .. }),
            "{refusal:?}"
        );
        assert_eq!(
            account.trial,
            refusal == SandboxStartRefusal::TrialAutoStopRequired,
            "{refusal:?}"
        );
        assert_eq!(snapshot.starts[0].ends_at, Some(at()), "{refusal:?}");
    }
}

#[tokio::test]
async fn supplied_retry_blocks_new_creates_only_until_its_boundary() {
    let backend = capabilities(Unimock::new((
        BoatAccountApiMock::account
            .some_call(matching!())
            .answers(&|_| Ok(facts())),
        SandboxLifecycleMock::create_sandbox
            .some_call(matching!(_))
            .answers(&|_, _| {
                Err(sandbox_interface::Error::SandboxStartRefused {
                    backend_id: "boat".to_owned(),
                    refusal: SandboxStartRefusal::PlanSize {
                        reason: PlanSizeRefusalReason::PlanRequired,
                    },
                })
            }),
    )));
    let harness = Harness::new(backend, (), Runtime::Start);
    let mut account = harness.snapshot().accounts[0].clone();
    account.state.retry_at = Some(at() + Duration::seconds(90));
    harness.save_account(&account);
    assert_eq!(
        harness.service.start(id(1), request()).await.unwrap(),
        StartOutcome::Waiting(WaitReason::RetryAt(at() + Duration::seconds(90)))
    );
    *harness.now.lock().unwrap() = at() + Duration::seconds(90);
    assert!(matches!(
        harness.service.start(id(1), request()).await.unwrap(),
        StartOutcome::Waiting(WaitReason::Refused(_))
    ));
}

#[tokio::test]
async fn trial_uses_two_hours_and_billing_waits_for_confirmed_recovery() {
    let backend = capabilities(Unimock::new((
        BoatAccountApiMock::account.stub(|each| {
            each.call(matching!())
                .answers(&|_| {
                    let mut facts = facts();
                    facts.billing_ready = false;
                    Ok(facts)
                })
                .once()
                .then()
                .answers(&|_| {
                    let mut facts = facts();
                    facts.trial = true;
                    Ok(facts)
                });
        }),
        SandboxLifecycleMock::create_sandbox
            .some_call(matching!(_))
            .answers(&|_, request| {
                assert_eq!(
                    request.lifetime,
                    SandboxLifetime::OneShot {
                        max_lifetime: std::time::Duration::from_secs(7200)
                    }
                );
                Err(sandbox_interface::Error::SandboxStartRefused {
                    backend_id: "boat".to_owned(),
                    refusal: SandboxStartRefusal::PlanSize {
                        reason: PlanSizeRefusalReason::TrialSizeNotAllowed,
                    },
                })
            }),
    )));
    let harness = Harness::new(backend, (), Runtime::Start);
    let mut account = harness.snapshot().accounts[0].clone();
    account.state.billing_required = true;
    harness.save_account(&account);
    assert_eq!(
        harness.service.start(id(1), request()).await.unwrap(),
        StartOutcome::Waiting(WaitReason::BillingRequired)
    );
    assert!(matches!(
        harness.service.start(id(1), request()).await.unwrap(),
        StartOutcome::Waiting(WaitReason::Refused(_))
    ));
    assert_eq!(
        harness.snapshot().accounts[0].state.refused_sizes,
        ["large"]
    );
}
