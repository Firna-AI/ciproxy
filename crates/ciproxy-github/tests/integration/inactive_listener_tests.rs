//! Local inactivity keeps exact completion bindings through their final ACK.

use std::sync::Arc;
use std::sync::atomic::AtomicI64;

use ciproxy_github::auth::tokens::RotatingTokens;
use ciproxy_github::listener::{DurableListener, ListenerOptions, ScaleSetListener};
use ciproxy_github::persistence::StoreAdapter;
use ciproxy_github::rest::http::RestClient;
use ciproxy_store_interface::github::DemandPhase;
use ciproxy_store_interface::scale_sets::ScaleSetState;
use diesel::prelude::*;
use diesel::sql_types;
use reqwest::{Client, Method, Url};
use serde_json::json;
use tokio::sync::watch;

use super::database::Database;
use super::fake_http::Server;
use super::support::{Factory, FixedJwt, FixedRuntime, Output, setup_with_label};

#[tokio::test]
async fn inactive_retired_binding_delivers_received_completion_and_final_ack_without_acquisition() {
    let database = Database::new();
    let target = setup_with_label(&*database.store, "ciproxy-completion");
    let server = Server::start(Some(database.store.clone())).await;
    let http = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let api = Url::parse(&format!("{}/api/v3/", server.origin)).unwrap();
    let runtime = Arc::new(FixedRuntime(AtomicI64::new(1_793_793_600)));
    let tokens = Arc::new(RotatingTokens::new(
        http.clone(),
        api.clone(),
        Arc::new(FixedJwt),
        runtime.clone(),
    ));
    let output = Arc::new(Output::default());
    let listener = DurableListener {
        store: Arc::new(StoreAdapter::new(database.store.clone(), runtime)),
        rest: Arc::new(RestClient::new(http, api, tokens)),
        scale_sets: Arc::new(Factory(server.origin.clone())),
        output: output.clone(),
    };
    let options = ListenerOptions {
        owner: "completion".into(),
        capacity: 2,
        max_polls: Some(1),
    };
    let (_stop, signal) = watch::channel(false);
    listener
        .run(target, options.clone(), signal.clone())
        .await
        .unwrap();
    let mut connection = PgConnection::establish(&database.url).unwrap();
    diesel::sql_query("UPDATE organization_activation SET admitted=FALSE,admin_approval_hold=TRUE WHERE organization_id=$1")
        .bind::<sql_types::Uuid,_>(target.organization_id).execute(&mut connection).unwrap();
    let bindings = database
        .store
        .listener_scale_sets(target.workspace_id, target.organization_id)
        .unwrap();
    assert_eq!(
        bindings.len(),
        1,
        "inactive received jobs must remain discoverable"
    );
    assert_eq!(bindings[0].scale_set.id, target.scale_set_id);
    assert_eq!(bindings[0].scale_set.github_id, 53);
    assert_eq!(bindings[0].state, ScaleSetState::Active);
    assert!(bindings[0].has_tracked_jobs);
    let available = json!({"messageType":"JobAvailable","jobId":"01a277ed-2171-562e-8431-03d0033029ab","jobDisplayName":"probe",
        "jobWorkflowRef":"shared/reusable/.github/workflows/spike.yml@refs/heads/main","ownerName":"customer","repositoryName":"private",
        "workflowRunId":31,"eventName":"workflow_dispatch","runnerRequestId":19,"queueTime":"0001-01-01T00:00:00Z",
        "scaleSetAssignTime":"2026-11-04T10:46:21.620395766Z","runnerAssignTime":"0001-01-01T00:00:00Z"});
    let mut completed = available.clone();
    completed["messageType"] = json!("JobCompleted");
    completed["runnerAssignTime"] = json!("2026-11-04T10:46:21.661245605Z");
    completed["finishTime"] = json!("2026-11-04T10:46:24.317465059Z");
    completed["runnerId"] = json!(15);
    completed["result"] = json!("succeeded");
    let mut assigned = available.clone();
    assigned["messageType"] = json!("JobAssigned");
    assigned["runnerRequestId"] = json!(0);
    *server.state.body_override.lock().unwrap() = Some(json!([available, assigned]).to_string());
    let result = listener.run(target, options.clone(), signal.clone()).await;
    assert!(
        matches!(result, Ok(())),
        "inactive active binding must refuse acquisition: {result:?}"
    );
    database
        .store
        .retire_dashboard_label(
            target.workspace_id,
            bindings[0].scale_set.label_id,
            "2026-11-04T12:00:00Z".parse().unwrap(),
        )
        .unwrap();
    let bindings = database
        .store
        .listener_scale_sets(target.workspace_id, target.organization_id)
        .unwrap();
    assert_eq!(bindings.len(), 1);
    assert_eq!(bindings[0].state, ScaleSetState::Draining);
    *server.state.body_override.lock().unwrap() = Some(json!([completed]).to_string());
    let result = listener.run(target, options, signal).await;
    assert!(
        matches!(result, Ok(())),
        "inactive completion must reach its final ACK: {result:?}"
    );
    assert_eq!(*server.state.poll_capacities.lock().unwrap(), vec![2, 0, 0]);
    assert_eq!(
        *server.state.acknowledgements.lock().unwrap(),
        vec![
            DemandPhase::Assigned,
            DemandPhase::Assigned,
            DemandPhase::Completed
        ]
    );
    assert_eq!(
        output
            .0
            .lock()
            .unwrap()
            .iter()
            .map(|job| job.phase)
            .collect::<Vec<_>>(),
        vec![DemandPhase::Assigned, DemandPhase::Completed]
    );
    assert_eq!(
        database
            .store
            .github_jobs(target.workspace_id, target.organization_id)
            .unwrap()[0]
            .phase,
        DemandPhase::Completed
    );
    assert_eq!(
        database
            .store
            .pending_github_jobs(
                target.workspace_id,
                target.organization_id,
                target.scale_set_id
            )
            .unwrap(),
        vec![]
    );
    assert_eq!(
        database
            .store
            .listener_scale_sets(target.workspace_id, target.organization_id)
            .unwrap(),
        vec![]
    );
    let binding = database
        .store
        .scale_set_binding(
            target.workspace_id,
            target.organization_id,
            target.scale_set_id,
        )
        .unwrap()
        .unwrap();
    assert_eq!(binding.scale_set.github_id, 53);
    assert!(!binding.has_tracked_jobs);
    let requests = server.state.requests.lock().unwrap();
    assert_eq!(
        requests
            .iter()
            .filter(|(_, path, _)| path.contains("acquirejobs"))
            .count(),
        0
    );
    assert_eq!(
        requests
            .iter()
            .filter(|(method, path, _)| *method == Method::DELETE
                && path.ends_with("sessions/session-1?api-version=6.0-preview"))
            .count(),
        3
    );
}
