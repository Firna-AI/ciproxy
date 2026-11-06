//! Reclaim and revocation boundaries use no disk, process, or network IO.

use std::sync::{Arc, Mutex};

use unimock::{MockFn, Unimock, matching};

use crate::boundaries::Cache;
use crate::cache::HttpCache;
use crate::cache_admin::{AdministrationMock, CacheFilesMock, Credential};
use crate::cache_authority::CacheAuthority;
use crate::error::Error;
use crate::fixtures::vm;
use crate::process::{CommandsMock, UnitState};
use ciproxy_protocol::identifiers::JobId;
use ciproxy_protocol::jobs::{JobScope, UnknownJobScopeReason};
use ciproxy_protocol::runtime::{CachePermissions, RuntimeCredentials};
use ciproxy_protocol::secret::Secret;

#[tokio::test]
async fn unavailable_cache_closes_handles_and_reclaims_space_before_acknowledgement() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let stopped = events.clone();
    let released = events.clone();
    let restarted = events.clone();
    let mock=Arc::new(Unimock::new((
  AdministrationMock::budget.some_call(matching!(80)).returns(Err(Error::CacheUnavailable)),
  CommandsMock::unit.some_call(matching!("ciproxy-cache.service")).returns(Ok(UnitState::Active)),
  CommandsMock::run.next_call(matching!((program,args,_) if *program=="systemctl" && args==&["stop","ciproxy-cache.service"])).answers_arc(Arc::new(move|_,_,_,_|{stopped.lock().unwrap().push("handles_closed");Ok(Vec::new())})),
  CacheFilesMock::release_occupied_space.next_call(matching!()).answers_arc(Arc::new(move |_|{released.lock().unwrap().push("space_released");Ok(())})),
  CommandsMock::run.next_call(matching!((program,args,_) if *program == "systemctl" && args == &["start","ciproxy-cache.service"])).answers_arc(Arc::new(move|_,_,_,_|{restarted.lock().unwrap().push("service_restarted");Ok(Vec::new())})),
 )));

    let cache = HttpCache::with_boundaries(mock.clone(), mock.clone(), mock);
    cache.budget(80).await.unwrap();
    assert_eq!(
        *events.lock().unwrap(),
        vec!["handles_closed", "space_released", "service_restarted"]
    );
}

#[tokio::test]
async fn failed_owned_reclaim_does_not_acknowledge_unreleased_space() {
    let mock = Arc::new(Unimock::new((
        AdministrationMock::budget
            .some_call(matching!(80))
            .returns(Err(Error::CacheUnavailable)),
        CommandsMock::unit
            .some_call(matching!("ciproxy-cache.service"))
            .returns(Ok(UnitState::Absent)),
        CacheFilesMock::release_occupied_space
            .some_call(matching!())
            .returns(Err(Error::CacheUnavailable)),
    )));
    let cache = HttpCache::with_boundaries(mock.clone(), mock.clone(), mock);
    assert_eq!(
        cache
            .budget(80)
            .await
            .map_err(|error| matches!(error, Error::CacheUnavailable)),
        Err(true)
    );
}

#[tokio::test]
async fn restored_vm_exit_revokes_its_persisted_job_without_a_runtime_handoff() {
    let revoked = Arc::new(Mutex::new(Vec::new()));
    let observed = revoked.clone();
    let mock = Arc::new(Unimock::new(
        AdministrationMock::revoke
            .some_call(matching!(_))
            .answers_arc(Arc::new(move |_, job| {
                observed.lock().unwrap().push(job.clone());
                Ok(())
            })),
    ));
    let cache = HttpCache::with_boundaries(mock.clone(), mock.clone(), mock);
    let mut restored = vm("restored", true);
    restored.job_facts = Some(crate::fixtures::trusted_facts());
    cache.release(&restored).await.unwrap();
    assert_eq!(
        *revoked.lock().unwrap(),
        vec![crate::fixtures::trusted_facts()]
    );
}

#[tokio::test]
async fn replay_reuses_live_credentials_and_registers_again_only_after_sidecar_loss() {
    let scope = JobScope::Unknown {
        reason: UnknownJobScopeReason::MissingToken,
    };
    let old = Credential {
        token: Secret::new("old"),
        scope: scope.clone(),
    };
    let new = Credential {
        token: Secret::new("new"),
        scope,
    };
    let recovered = old.clone();
    let mock = Arc::new(Unimock::new((
        AdministrationMock::recover.stub(|each| {
            let old = recovered.clone();
            let replacement = new.clone();
            each.call(matching!(_))
                .answers_arc(Arc::new(move |_, _| Ok(Some(old.clone()))))
                .n_times(2)
                .then()
                .answers(&|_, _| Ok(None))
                .once()
                .then()
                .answers_arc(Arc::new(move |_, _| Ok(Some(replacement.clone()))))
                .once();
        }),
        AdministrationMock::register
            .some_call(matching!(_))
            .returns(Ok(new)),
    )));
    let cache = HttpCache::with_boundaries(mock.clone(), mock.clone(), mock);
    let record = vm("running", true);
    let runtime = RuntimeCredentials {
        docker_data_bytes: 7,
        proxy_url: "http://bridge:8230/".into(),
        github_token: Secret::new("original"),
        cache_url: Secret::new("https://cache.example/"),
        results_url: Secret::new("https://results.example/"),
        permissions: CachePermissions {
            read: true,
            write: true,
        },
    };
    cache
        .handoff(
            &record,
            &JobId("job-1".into()),
            CacheAuthority::Waiting.handoff(true, runtime).unwrap(),
        )
        .await
        .unwrap();
    let facts = crate::fixtures::trusted_facts();
    for expected in ["old", "old", "new", "new"] {
        let (connection, _, data) = cache
            .prepare(&record.report.runner_id, &facts)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(connection.token, Secret::new(expected));
        assert_eq!(data, 7);
    }
}

#[tokio::test]
async fn periodic_reconciliation_failure_recovers_service_without_reclaiming_entries() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let released = events.clone();
    let commands = events.clone();
    let mock = Arc::new(Unimock::new((
        AdministrationMock::budget
            .each_call(matching!(_))
            .answers(&|_, _| Err(Error::CacheUnavailable)),
        CommandsMock::unit
            .each_call(matching!(_))
            .answers(&|_, _| Ok(UnitState::Active)),
        CacheFilesMock::release_occupied_space
            .each_call(matching!())
            .answers_arc(Arc::new(move |_| {
                released.lock().unwrap().push("reclaim".to_owned());
                Ok(())
            })),
        CommandsMock::run
            .each_call(matching!(_, _, _))
            .answers_arc(Arc::new(move |_, _, args, _| {
                commands.lock().unwrap().push(args[0].clone());
                Ok(Vec::new())
            })),
    )));
    let cache = Arc::new(HttpCache::with_boundaries(mock.clone(), mock.clone(), mock));
    let mut agent = crate::fixtures::controller(
        crate::state::MachineState::default(),
        Unimock::new(
            crate::boundaries::HostMock::hardware
                .some_call(matching!())
                .returns(Ok(crate::fixtures::hardware())),
        ),
    );
    agent.cache = cache.clone();
    crate::controller::Agent::reconcile(&agent).await.unwrap();
    assert_eq!(*events.lock().unwrap(), vec!["start"]);
    assert_eq!(agent.state.lock().await.runners.len(), 0);
    cache.budget(80).await.unwrap();
    assert_eq!(
        *events.lock().unwrap(),
        vec!["start", "stop", "reclaim", "start"]
    );
}
