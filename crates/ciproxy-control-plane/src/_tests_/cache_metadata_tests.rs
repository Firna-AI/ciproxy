//! The production metadata HTTP boundary authenticates workspace and job placement.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use ciproxy_github::rest::cache_metadata::CacheEntriesMock;
use ciproxy_github::runtime::RuntimeMock;
use ciproxy_protocol::cache_metadata::CACHE_METADATA_PATH;
use ciproxy_store_interface::github::DemandPhase;
use ciproxy_store_interface::own_runners::{OwnRunner, RoutingSnapshot};
use ciproxy_store_interface::runners::{Runner, RunnerPlace, RunnerState};
use ciproxy_store_interface::tokens::{Token, TokenSubject};
use serde_json::json;
use tower::ServiceExt;
use unimock::{MockFn, Unimock, matching};
use uuid::Uuid;

use crate::cache_metadata::records::MetadataRecordsMock;
use crate::cache_metadata::service::HostedCacheMetadata;
use crate::cache_metadata::transport::router;

use crate::snapshot::tests::fixtures;

#[tokio::test]
async fn metadata_http_uses_token_workspace_and_exact_assigned_machine() {
    for scenario in [
        "valid",
        "install",
        "revoked",
        "workspace",
        "machine",
        "runner",
        "completed",
    ] {
        let mut snapshot = assignment();
        let mut token = machine_token();
        match scenario {
            "install" => token.subject = TokenSubject::Install,
            "workspace" => snapshot.workspace_id = Uuid::from_u128(99),
            "machine" => token.subject = TokenSubject::Machine(Uuid::from_u128(99)),
            "runner" => snapshot.runners[0].runner.github_id = Some(99),
            "completed" => snapshot.jobs[0].phase = DemandPhase::Completed,
            _ => {}
        }
        let authentication = MetadataRecordsMock::authenticate.some_call(matching!((hash, at) if
            hash.as_bytes() == [62, 167, 112, 46, 136, 49, 130, 76, 181, 153, 150, 235, 255, 121, 117, 234, 88, 213, 241, 50, 191, 13, 147, 93, 243, 1, 25, 254, 238, 64, 16, 217]
            && *at == fixtures::at())).returns(Ok(if scenario == "revoked" { None } else { Some(token) }));
        let records = if ["install", "revoked"].contains(&scenario) {
            Arc::new(Unimock::new(authentication))
        } else {
            Arc::new(Unimock::new((
                authentication,
                MetadataRecordsMock::snapshot
                    .some_call(matching!((workspace) if *workspace == Uuid::from_u128(1)))
                    .returns(Ok(snapshot)),
            )))
        };
        let caches = if scenario == "valid" {
            Arc::new(Unimock::new(
                CacheEntriesMock::cache_entries
                    .some_call(matching!((2, "org/repo")))
                    .returns(Ok(vec![])),
            ))
        } else {
            Arc::new(Unimock::new(()))
        };
        let service = Arc::new(HostedCacheMetadata {
            records,
            caches,
            clock: Arc::new(Unimock::new(
                RuntimeMock::now
                    .some_call(matching!())
                    .returns(fixtures::at()),
            )),
        });
        let response = router(service).oneshot(Request::builder().method("POST").uri(CACHE_METADATA_PATH)
            .header("authorization","Bearer synthetic-machine-credential")
            .header("content-type","application/json")
            .body(Body::from(json!({"job_id":Uuid::from_u128(17).to_string(),"repository":"attacker/repo","ref":"refs/heads/attacker"}).to_string())).unwrap()).await.unwrap();
        assert_eq!(
            response.status(),
            if scenario == "valid" {
                StatusCode::OK
            } else {
                StatusCode::UNAUTHORIZED
            },
            "{scenario}"
        );
    }
}

fn machine_token() -> Token {
    Token {
        id: Uuid::from_u128(8),
        workspace_id: Uuid::from_u128(1),
        subject: TokenSubject::Machine(Uuid::from_u128(3)),
        created_at: fixtures::at(),
        expires_at: None,
        revoked_at: None,
    }
}

fn assignment() -> RoutingSnapshot {
    let mut input = fixtures::input();
    let mut job = fixtures::job(17, 2, 0);
    job.phase = DemandPhase::Started;
    job.runner_github_id = Some(71);
    job.scale_set_id = input.scale_sets[0].id;
    input.runners.push(OwnRunner {
        registration_scale_set_id: Some(2),
        requested_size: input.labels[0].size,
        slot_indices: vec![0],
        label_name: "ciproxy".into(),
        command_id: Uuid::from_u128(18),
        issued_at: fixtures::at(),
        acknowledged: true,
        confirmed_at: Some(fixtures::at()),
        runner: Runner {
            id: Uuid::from_u128(19),
            organization_id: job.organization_id,
            label_id: job.label_id,
            scale_set_id: job.scale_set_id,
            github_id: Some(71),
            state: RunnerState::Busy,
            size: ciproxy_protocol::resources::VmSize {
                vcpu: 2,
                memory_bytes: 8_000_000_000,
                disk_bytes: 80_000_000_000,
            },
            place: RunnerPlace::Machine {
                machine_id: Uuid::from_u128(3),
                first_slot: 0,
            },
        },
    });
    input.jobs.push(job);
    input
}
