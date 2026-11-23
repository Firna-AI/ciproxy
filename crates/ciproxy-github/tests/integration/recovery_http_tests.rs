//! Real HTTP proves saved group ownership, 204 absence, and typed recovery.

use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

use ciproxy_github::auth::tokens::RotatingTokens;
use ciproxy_github::error::Error;
use ciproxy_github::installation_state::{AppInstallationClient, InstallationApi};
use ciproxy_github::rest::GitHubRest;
use ciproxy_github::rest::http::RestClient;
use ciproxy_github::scale_sets::ScaleSetFactory;
use ciproxy_store_interface::organizations::{AppState, Organization, RepositoryAccess};
use reqwest::redirect::Policy;
use reqwest::{Client, Method, Url};

use super::fake_http::Server;
use super::support::{Factory, FixedJwt, FixedRuntime, id};

fn organization() -> Organization {
    Organization {
        id: id(5),
        workspace_id: id(1),
        github_id: 17,
        login: "customer".to_owned(),
        installation_id: 19,
        app_state: AppState::Active,
        repositories: RepositoryAccess::AllPrivate,
        removed_at: None,
        delete_after: None,
    }
}

fn rest_client(server: &Server) -> RestClient {
    let http = Client::builder().redirect(Policy::none()).build().unwrap();
    let api = Url::parse(&format!("{}/api/v3/", server.origin)).unwrap();
    let tokens = Arc::new(RotatingTokens::new(
        http.clone(),
        api.clone(),
        Arc::new(FixedJwt),
        Arc::new(FixedRuntime(AtomicI64::new(1_793_793_600))),
    ));
    RestClient::new(http, api, tokens)
}

#[tokio::test]
async fn owned_group_survives_rename_and_sdk_recovers_204_absence_with_typed_missing_signals() {
    let server = Server::start(None).await;
    let resource = &server.state.resource;
    resource.group_exists.store(true, Ordering::SeqCst);
    *resource.group_name.lock().unwrap() = "ciproxy-17".to_owned();
    let rest = rest_client(&server);
    let group = rest
        .restrict_runner_group(19, "customer", "ciproxy-3571-17", &[23], Some(47))
        .await
        .unwrap();
    let group_lists = || {
        server
            .state
            .requests
            .lock()
            .unwrap()
            .iter()
            .filter(|(method, path, _)| *method == Method::GET && path.contains("runner-groups?"))
            .count()
    };
    assert_eq!(
        (
            group,
            resource.group_creates.load(Ordering::SeqCst),
            group_lists(),
            resource.group_name.lock().unwrap().as_str(),
            resource.selected_repository_ids.lock().unwrap().as_slice(),
        ),
        (47, 0, 0, "ciproxy-3571-17", &[23][..])
    );
    *resource.group_name.lock().unwrap() = "owner-renamed".to_owned();
    let group = rest
        .restrict_runner_group(19, "customer", "ciproxy-3571-17", &[29], Some(47))
        .await
        .unwrap();
    assert_eq!(
        (
            group,
            resource.group_creates.load(Ordering::SeqCst),
            group_lists(),
            resource.group_name.lock().unwrap().as_str(),
            resource.selected_repository_ids.lock().unwrap().as_slice(),
        ),
        (47, 0, 0, "ciproxy-3571-17", &[29][..])
    );
    assert_eq!(
        rest.restrict_runner_group(19, "customer", "ciproxy-3571-17", &[23], None)
            .await
            .unwrap(),
        47
    );
    assert_eq!(
        server.state.resource.group_creates.load(Ordering::SeqCst),
        0
    );
    let sdk = Factory(server.origin.clone())
        .for_organization(&organization())
        .await
        .unwrap();
    server.state.resource.set_id.store(0, Ordering::SeqCst);
    assert!(matches!(
        sdk.ensure("ciproxy", 47, Some(53), &["ciproxy"]).await,
        Ok(83)
    ));
    let requests = server.state.requests.lock().unwrap().clone();
    assert_eq!(
        requests
            .iter()
            .filter(
                |(method, path, _)| *method == Method::GET && path.contains("runnerscalesets/53?")
            )
            .count(),
        1
    );
    assert_eq!(requests.iter().filter(|(method, path, _)| *method == Method::POST && path.contains("runnerscalesets?")).count(), 1);
    server
        .state
        .resource
        .missing_status
        .store(404, Ordering::SeqCst);
    assert!(matches!(
        sdk.open(83, "missing").await,
        Err(Error::ScaleSetMissing { scale_set_id: 83 })
    ));
    assert!(matches!(
        sdk.register(83, "runner-83").await,
        Err(Error::ScaleSetMissing { scale_set_id: 83 })
    ));
    server
        .state
        .resource
        .missing_status
        .store(0, Ordering::SeqCst);
    let queue = sdk.open(83, "before-delete").await.unwrap();
    server
        .state
        .resource
        .missing_status
        .store(404, Ordering::SeqCst);
    assert!(matches!(
        queue.poll(None, 2).await,
        Err(Error::ScaleSetMissing { scale_set_id: 83 })
    ));
    queue.close().await.unwrap();
    server.state.resource.set_id.store(53, Ordering::SeqCst);
    server.state.resource.set_group.store(99, Ordering::SeqCst);
    assert!(matches!(
        sdk.ensure("ciproxy", 47, Some(53), &["ciproxy"]).await,
        Err(Error::ScaleSetGroupChanged {
            scale_set_id: 53,
            expected_group: 47,
            actual_group: 99
        })
    ));
    assert_eq!(server.state.requests.lock().unwrap().iter().filter(|(method, path, _)| *method == Method::PATCH && path.contains("runnerscalesets")).count(), 0);
    assert!(
        matches!(
            sdk.rename_scale_set(99, "ciproxy-retired-absent", &[])
                .await,
            Ok(())
        ),
        "an absent retired scale set must not block preparation"
    );
}

#[tokio::test]
async fn group_lookup_without_saved_id_leaves_other_environment_untouched() {
    for other in ["ciproxy-17", "ciproxy-9001-17"] {
        let server = Server::start(None).await;
        let resource = &server.state.resource;
        resource.group_exists.store(true, Ordering::SeqCst);
        *resource.group_name.lock().unwrap() = other.to_owned();
        let group = rest_client(&server)
            .restrict_runner_group(19, "customer", "ciproxy-3571-17", &[23], None)
            .await
            .unwrap();
        let requests = server.state.requests.lock().unwrap();
        let writes: Vec<_> = requests
            .iter()
            .filter_map(|(method, path, body)| {
                let route = path.strip_prefix("/api/v3/orgs/customer/actions/runner-groups")?;
                (*method != Method::GET).then_some((method, route, body["name"].as_str()))
            })
            .collect();
        assert_eq!(
            (group, writes),
            (
                97,
                vec![
                    (&Method::POST, "", Some("ciproxy-3571-17")),
                    (&Method::PUT, "/97/repositories", None),
                ],
            ),
            "the group of another environment is named {other}"
        );
    }
}

#[tokio::test]
async fn installation_http_uses_app_jwt_and_keeps_transient_404_retryable() {
    let server = Server::start(None).await;
    let http = Client::builder().redirect(Policy::none()).build().unwrap();
    let api = Url::parse(&format!("{}/api/v3/", server.origin)).unwrap();
    let state = AppInstallationClient::new(http, api, Arc::new(FixedJwt));
    server
        .state
        .resource
        .installation_hidden
        .store(true, Ordering::SeqCst);
    assert!(matches!(
        state.for_organization("customer").await,
        Err(Error::InstallationPending)
    ));
    server
        .state
        .resource
        .installation_hidden
        .store(false, Ordering::SeqCst);
    server
        .state
        .resource
        .suspended
        .store(true, Ordering::SeqCst);
    let current = state.for_organization("customer").await.unwrap();
    assert_eq!(
        (current.id, current.account.id, current.suspended),
        (19, 17, true)
    );
    let identity = state.by_id(19).await.unwrap();
    assert_eq!((identity.id, identity.account.id), (19, 17));
    assert_eq!(
        server.state.resource.app_jwt_reads.load(Ordering::SeqCst),
        3
    );
    assert_eq!(server.state.tokens.load(Ordering::SeqCst), 0);
}
