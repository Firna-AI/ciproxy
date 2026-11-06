//! Construct dashboard authority before the hosted gateway and GitHub services.

use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use ciproxy_api::access::AccessService;
use ciproxy_api::crypto::AeadCredentialCipher;
use ciproxy_api::identity::http::{GithubOAuthClient, OAuthConfig};
use ciproxy_api::installations::GithubInstallationAdmin;
use ciproxy_api::maintenance::{ApiMaintenance, Maintenance};
use ciproxy_api::runtime::SystemRuntime;
use ciproxy_api::state::{ApiConfig, ApiState};
use ciproxy_store_interface::store::Store;
use reqwest::{Client, Url, redirect};

use crate::capacity::ReportedCapacity;
use crate::composition::Hosted;
use crate::config::Config;
use crate::error::{ErrorResultExt, Result};

pub(crate) struct ApiComposition {
    pub(crate) hosted: Arc<Hosted>,
    pub(crate) router: Router,
    pub(crate) maintenance: Arc<dyn ApiMaintenance>,
}

pub(crate) fn compose(config: &Config, store: Arc<dyn Store>) -> Result<ApiComposition> {
    let runtime = Arc::new(SystemRuntime);
    let http = Client::builder()
        .redirect(redirect::Policy::none())
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(90))
        .build()
        .internal()?;
    let api = Url::parse("https://api.github.com/").internal()?;
    let identity = Arc::new(GithubOAuthClient::new(
        http.clone(),
        OAuthConfig {
            client_id: config.github_oauth_client_id.clone(),
            client_secret: config.secret("github-oauth-secret")?,
            callback_url: format!(
                "{}/api/auth/github/callback",
                config.public_url.trim_end_matches('/')
            ),
            github: Url::parse("https://github.com/").internal()?,
            api: api.clone(),
        },
        runtime.clone(),
    ));
    let access = Arc::new(AccessService::new(
        store.clone(),
        identity,
        Arc::new(AeadCredentialCipher::new(&config.session_key()?)),
        runtime.clone(),
    ));
    let hosted = Arc::new(Hosted::new(
        config,
        store.clone(),
        access.clone(),
        access.clone(),
    )?);
    let state = ApiState {
        overflow: hosted.overflow.clone(),
        config: ApiConfig {
            public_origin: config.public_url.trim_end_matches('/').to_owned(),
            github_install_url: format!(
                "https://github.com/apps/{}/installations/new",
                config.github_app_slug
            ),
            download_url: config.download_url.clone(),
            image_disk_bytes: config.image_disk_bytes,
            build: config.build_id.clone(),
        },
        store,
        access,
        gateway: hosted.gateway.clone(),
        github: hosted.app.clone(),
        scale_sets: hosted.scale_sets.clone(),
        installations: Arc::new(GithubInstallationAdmin::new(
            http,
            api,
            hosted.signer.clone(),
        )),
        capacity: Arc::new(ReportedCapacity),
        runtime,
    };
    let router = hosted.router(ciproxy_api::router::router(state.clone()));
    Ok(ApiComposition {
        hosted,
        router,
        maintenance: Arc::new(Maintenance { state }),
    })
}
