//! Real-store page fixture with the production billing and metering services.

use std::sync::Arc;

use axum::Extension;
use axum::Router;
use axum::body::Body;
use axum::http::{Request, Response, header};
use chrono::{DateTime, Utc};
use ciproxy_api::capacity::{Capacity, MachineCapacity, MachineCapacityMock};
use ciproxy_api::pages::{PageServices, stripe_router};
use ciproxy_api::router::router_with_agent_releases;
use ciproxy_api::runtime::Runtime;
use ciproxy_billing::alerts::HostedBillingAlerts;
use ciproxy_billing::service::runtime::{BillingService, Redirects};
use ciproxy_billing::stripe::api::StripeApi;
use ciproxy_billing::webhook::SigningSecret;
use ciproxy_metering::report::catalog::{Alternative, PriceList, SizePrice};
use ciproxy_metering::service::StoredMetering;
use ciproxy_protocol::resources::Slots;
use ciproxy_store_interface::billing::state::{BillingState, SubscriptionState, Tier};
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Timestamptz};
use hmac::{Hmac, KeyInit, Mac};
use rust_decimal::Decimal;
use serde_json::Value;
use sha2::Sha256;
use tower::ServiceExt;
use unimock::{MockFn, Unimock, matching};
use uuid::Uuid;

use super::fixture::{Browser, Fixture};

pub(super) struct Pages {
    pub(super) fixture: Fixture,
    pub(super) services: Arc<PageServices>,
}

impl Pages {
    pub(super) fn new(stripe: Arc<dyn StripeApi>) -> Self {
        let fixture = Fixture::new();
        let start = at("2026-11-01T00:00:00Z");
        let mut connection = PgConnection::establish(&fixture.database.url).unwrap();
        for statement in [
            "UPDATE billing_accounts SET created_at=$1 WHERE workspace_id=$2",
            "UPDATE workspaces SET metering_started_at=$1 WHERE id=$2",
            "UPDATE metering_plan_costs SET starts_at=$1 WHERE workspace_id=$2",
            "UPDATE metering_periods SET starts_at=$1 WHERE workspace_id=$2",
        ] {
            diesel::sql_query(statement)
                .bind::<Timestamptz, _>(start)
                .bind::<diesel::sql_types::Uuid, _>(fixture.workspace)
                .execute(&mut connection)
                .unwrap();
        }
        let billing = Arc::new(BillingService::new(
            fixture.database.store.clone(),
            stripe,
            fixture.runtime.clone(),
            Arc::new(HostedBillingAlerts),
            SigningSecret::new("whsec_m18_offline_only".into()),
            Redirects {
                success_url:
                    "https://app.ciproxy.example/billing/return?session_id={CHECKOUT_SESSION_ID}"
                        .into(),
                cancel_url: "https://app.ciproxy.example/plans".into(),
                portal_return_url: "https://app.ciproxy.example/billing".into(),
            },
        ));
        let services = Arc::new(PageServices {
            boat_root_sizing: None,
            billing,
            metering: Arc::new(StoredMetering::new(fixture.database.store.clone())),
            price_lists: Some(catalogs()),
            paid_review: None,
            overflow: None,
        });
        Self { fixture, services }
    }

    pub(super) fn application(&self) -> Router {
        Router::new()
            .nest(
                "/api",
                router_with_agent_releases(
                    self.fixture.state.clone(),
                    self.fixture.releases.clone(),
                )
                .layer(Extension(self.services.clone())),
            )
            .merge(stripe_router(
                self.fixture.state.clone(),
                self.services.billing.clone(),
            ))
    }

    pub(super) async fn request(
        &self,
        browser: Option<&Browser>,
        method: &str,
        suffix: &str,
        body: Option<Value>,
    ) -> Response<Body> {
        let request = self.build(browser, method, suffix, body);
        let response = self.application().oneshot(request).await.unwrap();
        assert_eq!(response.headers()["x-ciproxy-build"], "http-test-build");
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
        response
    }

    pub(super) fn build(
        &self,
        browser: Option<&Browser>,
        method: &str,
        suffix: &str,
        body: Option<Value>,
    ) -> Request<Body> {
        let mut request = Request::builder()
            .method(method)
            .uri(format!("/api{}", self.fixture.path(suffix)));
        if let Some(browser) = browser {
            request = request.header(header::COOKIE, format!("__Host-ciproxy={}", browser.cookie));
            if method != "GET" {
                request = request
                    .header(header::ORIGIN, &self.fixture.state.config.public_origin)
                    .header("x-ciproxy-csrf", &browser.csrf);
            }
        }
        match body {
            Some(body) => request
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
            None => request.body(Body::empty()).unwrap(),
        }
    }

    pub(super) fn state(&self, tier: Tier) -> BillingState {
        let mut state = BillingState::free(self.fixture.workspace, at("2026-11-01T00:00:00Z"));
        if tier != Tier::Free {
            state.tier = tier;
            state.customer_id = Some("cus_m18".into());
            state.subscription_id = Some("sub_m18".into());
            state.status = SubscriptionState::Active;
            state.paid_period_start = Some(at("2026-11-01T00:00:00Z"));
            state.paid_period_end = Some(at("2026-12-01T00:00:00Z"));
        }
        state
    }

    pub(super) fn save(&self, state: &BillingState) {
        let mut session = self
            .fixture
            .database
            .store
            .try_lock_billing(self.fixture.workspace)
            .unwrap()
            .unwrap();
        session
            .commit(Some(state), None, true, false, self.fixture.runtime.now())
            .unwrap();
    }

    pub(super) fn starts(&self, count: i64) {
        let mut connection = PgConnection::establish(&self.fixture.database.url).unwrap();
        diesel::sql_query("INSERT INTO metering_start_totals(workspace_id,taken_at,starts) VALUES ($1,'2026-11-02T12:00:00Z',$2) ON CONFLICT(workspace_id,taken_at) DO UPDATE SET starts=EXCLUDED.starts")
            .bind::<diesel::sql_types::Uuid, _>(self.fixture.workspace).bind::<BigInt, _>(count).execute(&mut connection).unwrap();
    }

    pub(super) fn registration(&self, machine: Uuid) {
        let mut connection = PgConnection::establish(&self.fixture.database.url).unwrap();
        diesel::sql_query("UPDATE machines SET registered_at=$1 WHERE id=$2 AND workspace_id=$3")
            .bind::<Timestamptz, _>(self.fixture.runtime.now())
            .bind::<diesel::sql_types::Uuid, _>(machine)
            .bind::<diesel::sql_types::Uuid, _>(self.fixture.workspace)
            .execute(&mut connection)
            .unwrap();
    }
}

pub(super) fn at(value: &str) -> DateTime<Utc> {
    value.parse().unwrap()
}

pub(super) fn catalogs() -> [PriceList; 2] {
    [Alternative::Blacksmith, Alternative::Github].map(|alternative| PriceList {
        alternative,
        dated: at("2026-11-01T00:00:00Z").date_naive(),
        sizes: vec![SizePrice {
            vcpus: 2,
            memory_gb: 8,
            minute_usd: if alternative == Alternative::Blacksmith {
                Decimal::new(6, 3)
            } else {
                Decimal::new(8, 3)
            },
        }],
    })
}

pub(super) fn signature(raw: &[u8]) -> String {
    let time = at("2026-11-06T12:00:00Z").timestamp();
    let mut mac = Hmac::<Sha256>::new_from_slice(b"whsec_m18_offline_only").unwrap();
    mac.update(format!("{time}.").as_bytes());
    mac.update(raw);
    format!("t={time},v1={}", hex::encode(mac.finalize().into_bytes()))
}

impl ciproxy_billing::clock::Clock for super::runtime_inputs::Inputs {
    fn now(&self) -> DateTime<Utc> {
        Runtime::now(self)
    }
    fn command_id(&self) -> Uuid {
        Runtime::identifier(self)
    }
}

pub(super) fn capacity() -> Arc<dyn MachineCapacity> {
    Arc::new(Unimock::new(
        MachineCapacityMock::capacity
            .each_call(matching!(_))
            .returns(Capacity {
                slots: Slots(6),
                job_disk_bytes: Some(180_000_000_000),
                image_disk_bytes: Some(58_300_000_000),
            }),
    ))
}
