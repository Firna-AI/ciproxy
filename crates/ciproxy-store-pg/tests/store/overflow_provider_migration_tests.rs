//! Forward migration of saved Boat account, runner, and metering facts.

use chrono::Duration;
use ciproxy_protocol::resources::VmSize;
use ciproxy_schema::metering::metering_job_cursors;
use ciproxy_schema::migrations::MIGRATIONS;
use ciproxy_schema::overflow::overflow_accounts;
use ciproxy_store_interface::metering::facts::{Completion, JobPlace, JobSource, UsageJob};
use ciproxy_store_interface::metering::history::OverflowCost;
use ciproxy_store_interface::metering::totals::{
    AccountedJob, AccountedOverflow, DailyTotal, LocationTotals, SizeTotal,
};
use ciproxy_store_interface::overflow::provider::OverflowProvider;
use ciproxy_store_interface::overflow_provider_accounts::OverflowProviderAccount;
use ciproxy_store_interface::runners::RunnerPlace;
use ciproxy_store_interface::secrets::EncryptedOverflowKey;
use ciproxy_store_interface::store::StoreFactory;
use ciproxy_store_pg::factory::PgStoreFactory;
use diesel::connection::SimpleConnection;
use diesel::migration::MigrationVersion;
use diesel::prelude::*;
use diesel_migrations::MigrationHarness;
use rust_decimal::Decimal;
use serde_json::{Value, json};
use uuid::Uuid;

use super::support::{TestDatabase, at};

#[test]
fn migrates_boat_accounts_and_metering_facts_with_their_data() {
    let database = TestDatabase::new();
    let mut connection = PgConnection::establish(&database.url).unwrap();
    let pending = connection.pending_migrations(MIGRATIONS).unwrap();
    let provider_records = MigrationVersion::from("202611231528");
    for migration in pending
        .iter()
        .take_while(|migration| migration.name().version() < provider_records)
    {
        connection.run_migration(migration.as_ref()).unwrap();
    }
    connection
        .batch_execute(include_str!("overflow_provider_migration.sql"))
        .unwrap();
    let store = PgStoreFactory.connect(&database.url).unwrap();
    let workspace = Uuid::from_u128(1);
    let organization = Uuid::from_u128(2);
    let key = Uuid::from_u128(5);
    let runner_id = Uuid::from_u128(6);
    let provider = OverflowProvider::Boat;
    let day = at().date_naive();
    let size = VmSize {
        vcpu: 2,
        memory_bytes: 8_000_000_000,
        disk_bytes: 110_000_000_000,
    };

    assert_eq!(
        store.overflow_provider_accounts(workspace).unwrap(),
        vec![OverflowProviderAccount {
            id: key,
            provider,
            account_name: "Legacy account".to_owned(),
            connected_at: at(),
            retired_at: None,
            rejected_at: None,
        }]
    );
    assert_eq!(
        store.encrypted_overflow_key(workspace, key).unwrap(),
        Some(EncryptedOverflowKey::new(vec![1, 2]))
    );
    let state_json: String = overflow_accounts::table
        .find(key)
        .select(overflow_accounts::state_json)
        .first(&mut connection)
        .unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(&state_json).unwrap(),
        json!({
            "provider": "boat",
            "wallet": "personal",
            "sizes": [],
            "trial": true,
            "billing_required": false,
            "refused_sizes": []
        })
    );
    assert_eq!(
        store.runners(workspace, organization).unwrap()[0].place,
        RunnerPlace::Overflow {
            provider: OverflowProvider::Boat,
            key_id: Some(key),
            sandbox_id: "saved-sandbox".to_owned(),
            size_name: "saved-size".to_owned(),
        }
    );

    let cost = OverflowCost {
        runner_id,
        provider,
        starts_at: at(),
        ends_at: Some(at() + Duration::minutes(10)),
        start_usd: Decimal::new(7, 2),
        minute_usd: Decimal::new(2, 2),
    };
    let accounting = store.overflow_accounting(workspace).unwrap();
    assert_eq!(accounting.costs, vec![cost.clone()]);
    assert_eq!(
        accounting.accounted,
        vec![AccountedOverflow {
            cost,
            observed_at: at() + Duration::minutes(15),
        }]
    );
    assert_eq!(
        store
            .savings_snapshot(workspace, day, day.succ_opt().unwrap())
            .unwrap()
            .days,
        vec![DailyTotal {
            day,
            sizes: vec![SizeTotal {
                size,
                machines: LocationTotals::default(),
                overflow: [(
                    OverflowProvider::Boat,
                    LocationTotals {
                        starts: 1,
                        minutes: Decimal::new(75, 1),
                    }
                )]
                .into(),
            }],
            overflow_usd: [(OverflowProvider::Boat, Decimal::new(27, 2))].into(),
        }]
    );
    let cursor_json: Value = metering_job_cursors::table
        .select(metering_job_cursors::facts)
        .first(&mut connection)
        .unwrap();
    assert_eq!(
        cursor_json["place"],
        json!({"overflow": {"provider": "boat"}})
    );
    assert_eq!(
        store.workspace_usage(workspace).unwrap().accounted_jobs,
        vec![AccountedJob {
            job: UsageJob {
                source: JobSource::Actions,
                id: Uuid::from_u128(8),
                organization_id: organization,
                started_at: at(),
                completion: Completion::Running,
                size,
                place: JobPlace::Overflow {
                    provider: OverflowProvider::Boat
                },
            },
            observed_at: at() + Duration::seconds(450),
        }]
    );
}
