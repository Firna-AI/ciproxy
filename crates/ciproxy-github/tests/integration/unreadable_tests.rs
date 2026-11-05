//! Unreadable known work remains on GitHub instead of disappearing during decode.

use std::sync::Arc;
use std::sync::atomic::AtomicI64;

use ciproxy_github::auth::tokens::RotatingTokens;
use ciproxy_github::error::Error;
use ciproxy_github::listener::{DurableListener, ListenerOptions, ScaleSetListener};
use ciproxy_github::persistence::StoreAdapter;
use ciproxy_github::rest::http::RestClient;
use reqwest::redirect::Policy;
use reqwest::{Client, Method, Url};
use tokio::sync::watch;

use super::database::Database;
use super::fake_http::Server;
use super::support::{Factory, FixedJwt, FixedRuntime, Output, setup};

#[tokio::test]
async fn malformed_known_job_and_unreadable_batch_remain_unacknowledged() {
    for body in [
        r#"[{"messageType":"JobAssigned","jobId":"not-a-uuid","scaleSetAssignTime":"2026-11-04T10:46:21.620395766Z","ownerName":"customer","repositoryName":"private","workflowRunId":31}]"#,
        "[unreadable",
    ] {
        let database = Database::new();
        let target = setup(&*database.store);
        let server = Server::start(Some(database.store.clone())).await;
        *server.state.body_override.lock().unwrap() = Some(body.to_owned());
        let http = Client::builder().redirect(Policy::none()).build().unwrap();
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
        let (_stop, signal) = watch::channel(false);
        let result = listener
            .run(
                target,
                ListenerOptions {
                    owner: "bad-envelope".to_owned(),
                    capacity: 2,
                    max_polls: Some(1),
                },
                signal,
            )
            .await;
        assert!(matches!(result, Err(Error::Internal(_))), "{body}");
        assert_eq!(
            database
                .store
                .github_jobs(target.workspace_id, target.organization_id)
                .unwrap(),
            vec![],
            "{body}"
        );
        assert_eq!(output.0.lock().unwrap().len(), 0, "{body}");
        assert_eq!(server.state.requests.lock().unwrap().iter().filter(|(method, path, _)| *method == Method::DELETE && path.starts_with("/queue/")).count(), 0, "{body}");
        assert_eq!(server.state.requests.lock().unwrap().iter().filter(|(method, path, _)| *method == Method::DELETE && path.contains("/sessions/")).count(), 1, "{body}");
    }
}
