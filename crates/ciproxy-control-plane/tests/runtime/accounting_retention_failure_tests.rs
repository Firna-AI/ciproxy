//! Failed and stale refreshes defer retention without stopping other workspaces.

use std::sync::{Arc, Mutex};

use chrono::Duration;
use ciproxy_control_plane::accounting::Accounting;
use ciproxy_metering::service::StoredMetering;
use uuid::Uuid;

use super::accounting_refresh_faults::{Fault, RefreshFault};
use super::accounting_retention_support::{accounting, expired_demand, organization};
use super::support::{Fixture, at};

#[tokio::test]
async fn accounting_refresh_failure_or_stale_data_keeps_retention_local_and_retries() {
    for fault in [
        Fault::Error,
        Fault::Panic,
        Fault::StaleJob,
        Fault::StaleSandbox,
    ] {
        let fixture = Fixture::new();
        let store = &fixture.store;
        let healthy = organization(&fixture, Uuid::from_u128(u128::MAX), 100);
        let old = expired_demand(&fixture);
        store
            .record_app_removal(fixture.workspace, fixture.organization, at())
            .unwrap();
        store
            .record_app_removal(healthy.workspace_id, healthy.id, at())
            .unwrap();
        let removed = store
            .organization(fixture.workspace, fixture.organization)
            .unwrap();
        let workspace = store.workspace(fixture.workspace).unwrap();
        let labels = store.labels(fixture.workspace).unwrap();
        let calls = Arc::new(Mutex::new(Vec::new()));
        let (worker, now) = accounting(
            &fixture,
            Arc::new(RefreshFault {
                service: StoredMetering::new(store.clone()),
                failed: fixture.workspace,
                fault,
                calls: calls.clone(),
            }),
        );
        *now.lock().unwrap() = at() + Duration::days(30);
        worker.pass().await;
        assert_eq!(
            *calls.lock().unwrap(),
            vec![fixture.workspace, healthy.workspace_id],
            "{fault:?}"
        );
        assert_eq!(
            store.workspace(fixture.workspace).unwrap(),
            workspace,
            "{fault:?}"
        );
        assert_eq!(
            store
                .organization(fixture.workspace, fixture.organization)
                .unwrap(),
            removed,
            "{fault:?}"
        );
        assert_eq!(
            store.labels(fixture.workspace).unwrap(),
            labels,
            "{fault:?}"
        );
        assert_eq!(
            store
                .github_jobs(fixture.workspace, fixture.organization)
                .unwrap(),
            vec![old],
            "{fault:?}"
        );
        assert_eq!(
            store.workspace(healthy.workspace_id).unwrap(),
            None,
            "{fault:?}"
        );
        assert_eq!(
            store
                .organization(healthy.workspace_id, healthy.id)
                .unwrap(),
            None,
            "{fault:?}"
        );
        let (retry, now) = accounting(&fixture, Arc::new(StoredMetering::new(store.clone())));
        *now.lock().unwrap() = at() + Duration::days(30);
        retry.pass().await;
        assert_eq!(
            store.workspace(fixture.workspace).unwrap(),
            None,
            "{fault:?}"
        );
        assert_eq!(
            store
                .organization(fixture.workspace, fixture.organization)
                .unwrap(),
            None,
            "{fault:?}"
        );
    }
}
