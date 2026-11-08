//! Bound production routes with real access, billing, metering, and persistence.

use std::sync::Arc;

use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use ciproxy_api::runtime::Runtime;
use ciproxy_billing::clock::Clock;
use ciproxy_billing::service::runtime::Redirects;
use ciproxy_billing::stripe::wire::types::Subscription;
use ciproxy_billing::webhook::SigningSecret;
use ciproxy_control_plane::pages::{PageComposition, PageConfig, StripeSettings};
use ciproxy_metering::report::catalog::{Alternative, PriceList, SizePrice};
use ciproxy_metering::service::StoredMetering;
use ciproxy_store_interface::billing::state::{BillingState, SubscriptionState, Tier};
use ciproxy_store_interface::jobs::{Job, JobState};
use diesel::prelude::*;
use diesel::sql_types::Timestamptz;
use hmac::{Hmac, KeyInit, Mac};
use reqwest::{Client, Method, Response};
use rust_decimal::Decimal;
use serde_json::Value;
use sha2::Sha256;
use uuid::Uuid;

use super::fake::Fake;
use super::fixture::{Browser, Fixture};
use super::operator_support::OperatorEvents;
use super::runtime_inputs::Inputs;
use super::support_host::{Server, host};

pub(super) struct Dashboard {
    pub(super) fixture: Fixture,
    pub(super) pages: PageComposition,
    pub(super) fake: Fake,
    pub(super) origin: String,
    pub(super) client: Client,
    pub(super) alerts: Arc<OperatorEvents>,
    server: Server,
}

impl Dashboard {
    pub(super) async fn restart(&mut self) {
        self.server.0.abort();
        let _ = (&mut self.server.0).await;
        let pages = composition(&self.fixture, &self.fake, self.alerts.clone()).await;
        let previous = std::mem::replace(&mut self.pages, pages);
        tokio::task::spawn_blocking(move || drop(previous))
            .await
            .unwrap();
        let (origin, server) = host(&self.fixture, &self.pages).await;
        self.origin = origin;
        self.server = server;
    }

    pub(super) async fn close(mut self) {
        self.server.0.abort();
        let _ = (&mut self.server.0).await;
        let Self { pages, fake, .. } = self;
        tokio::task::spawn_blocking(move || drop((pages, fake)))
            .await
            .unwrap();
    }

    pub(super) async fn new() -> Self {
        let fixture = Fixture::new();
        let mut connection = PgConnection::establish(&fixture.database.url).unwrap();
        for statement in [
            "UPDATE billing_accounts SET created_at=$1 WHERE workspace_id=$2",
            "UPDATE workspaces SET metering_started_at=$1 WHERE id=$2",
            "UPDATE metering_plan_costs SET starts_at=$1 WHERE workspace_id=$2",
            "UPDATE metering_periods SET starts_at=$1 WHERE workspace_id=$2",
        ] {
            diesel::sql_query(statement)
                .bind::<Timestamptz, _>(at("2026-11-01T00:00:00Z"))
                .bind::<diesel::sql_types::Uuid, _>(fixture.workspace)
                .execute(&mut connection)
                .unwrap();
        }
        let mut subscription: Subscription = serde_json::from_str(include_str!(
            "../../../ciproxy-billing/tests/_tests_/fixtures/subscription.json"
        ))
        .unwrap();
        subscription.metadata.workspace_id = Some(fixture.workspace);
        subscription.metadata.command_id = None;
        subscription.billing_cycle_anchor = Some(at("2026-10-31T12:00:00Z").timestamp());
        let fake = Fake::new(subscription);
        let alerts = Arc::new(OperatorEvents::default());
        let pages = composition(&fixture, &fake, alerts.clone()).await;
        let (origin, server) = host(&fixture, &pages).await;
        Self {
            fixture,
            pages,
            fake,
            origin,
            server,
            client: Client::builder().no_proxy().build().unwrap(),
            alerts,
        }
    }

    pub(super) async fn request(
        &self,
        browser: Option<&Browser>,
        method: Method,
        suffix: &str,
        body: Option<Value>,
    ) -> Response {
        let mut request = self.client.request(
            method.clone(),
            format!("{}/api{}", self.origin, self.fixture.path(suffix)),
        );
        if let Some(browser) = browser {
            request = request.header("Cookie", format!("__Host-ciproxy={}", browser.cookie));
            if method != Method::GET {
                request = request
                    .header("Origin", &self.fixture.state.config.public_origin)
                    .header("x-ciproxy-csrf", &browser.csrf);
            }
        }
        if let Some(body) = body {
            request = request.json(&body);
        }
        let response = request.send().await.unwrap();
        assert_eq!(response.headers()["x-ciproxy-build"], "http-test-build");
        assert_eq!(response.headers()["Cache-Control"], "no-store");
        response
    }

    pub(super) fn save(&self, state: &BillingState) {
        self.fixture
            .database
            .store
            .try_lock_billing(self.fixture.workspace)
            .unwrap()
            .unwrap()
            .commit(
                Some(state),
                None,
                true,
                false,
                Runtime::now(&*self.fixture.runtime),
            )
            .unwrap();
    }

    pub(super) fn paid(&self) -> BillingState {
        let remote = self.fake.state.lock().unwrap();
        let mut state = BillingState::free(self.fixture.workspace, at("2026-11-01T00:00:00Z"));
        state.tier = Tier::Team;
        state.add_ons.start_blocks = 2;
        state.add_ons.machines = 1;
        state.customer_id = Some(remote.subscription.customer.clone());
        state.subscription_id = Some(remote.subscription.id.clone());
        state.status = SubscriptionState::Active;
        state.paid_period_start =
            DateTime::from_timestamp(remote.subscription.items.data[0].current_period_start, 0);
        state.paid_period_end =
            DateTime::from_timestamp(remote.subscription.items.data[0].current_period_end, 0);
        state
    }

    pub(super) fn queued_job(&self) -> Uuid {
        let scale = self
            .fixture
            .database
            .store
            .scale_sets(self.fixture.workspace, self.fixture.organization.id)
            .unwrap()
            .remove(0);
        let job = Job {
            id: Uuid::new_v4(),
            organization_id: self.fixture.organization.id,
            github_id: 8001,
            label_id: scale.label_id,
            scale_set_id: scale.id,
            runner_id: None,
            name: "composed-build".into(),
            workflow: "CI".into(),
            repository: "customer/private".into(),
            git_ref: Some("refs/heads/main".into()),
            state: JobState::Queued,
            result: None,
            queued_at: Runtime::now(&*self.fixture.runtime),
            started_at: None,
            completed_at: None,
            routing_reason: None,
            cache_source: None,
            machine_handled_cache_counts: None,
        };
        self.fixture
            .database
            .store
            .insert_job(self.fixture.workspace, &job)
            .unwrap();
        job.id
    }
}

pub(super) async fn composition(
    fixture: &Fixture,
    fake: &Fake,
    alerts: Arc<OperatorEvents>,
) -> PageComposition {
    let billing = PageComposition::billing(
        fixture.database.store.clone(),
        fixture.runtime.clone(),
        alerts,
        StripeSettings {
            endpoint: fake.url.clone(),
            secret_key: "sk_test_offline_only".into(),
            signing_secret: SigningSecret::new("whsec_offline_only".into()),
            redirects: Redirects {
                success_url:
                    "https://app.ciproxy.example/billing/return?session_id={CHECKOUT_SESSION_ID}"
                        .into(),
                cancel_url: "https://app.ciproxy.example/plans".into(),
                portal_return_url: "https://app.ciproxy.example/billing".into(),
            },
        },
    )
    .await
    .unwrap();
    PageComposition::new(
        fixture.database.store.clone(),
        billing,
        Arc::new(StoredMetering::new(fixture.database.store.clone())),
        None,
        PageConfig {
            boat_root_sizing: None,
            price_lists: [Alternative::Blacksmith, Alternative::Github].map(|alternative| {
                PriceList {
                    alternative,
                    dated: at("2026-11-07T00:00:00Z").date_naive(),
                    sizes: vec![SizePrice {
                        vcpus: 2,
                        memory_gb: 8,
                        minute_usd: Decimal::new(4, 3),
                    }],
                }
            }),
        },
    )
}

pub(super) fn at(value: &str) -> DateTime<Utc> {
    value.parse().unwrap()
}

pub(super) async fn json(response: Response, status: StatusCode) -> Value {
    let actual = response.status();
    let body = response.text().await.unwrap();
    assert_eq!(actual, status, "synthetic response: {body}");
    serde_json::from_str(&body).unwrap()
}

pub(super) fn signature(raw: &[u8], now: DateTime<Utc>) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(b"whsec_offline_only").unwrap();
    mac.update(format!("{}.", now.timestamp()).as_bytes());
    mac.update(raw);
    format!(
        "t={},v1={}",
        now.timestamp(),
        hex::encode(mac.finalize().into_bytes())
    )
}

impl Clock for Inputs {
    fn now(&self) -> DateTime<Utc> {
        Runtime::now(self)
    }
    fn command_id(&self) -> Uuid {
        self.identifier()
    }
}
