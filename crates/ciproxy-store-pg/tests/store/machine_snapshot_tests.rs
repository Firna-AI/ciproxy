//! Machine aggregate reads overlapping one committed report generation.

use std::thread;
use std::time::{Duration, Instant};

use chrono::TimeDelta;
use ciproxy_protocol::reports::DiskReport;
use ciproxy_schema::machines::{machine_disks, machine_reports, machines};
use ciproxy_store_interface::gateway::MachineSnapshot;
use ciproxy_store_interface::store::{Store, StoreFactory};
use ciproxy_store_pg::factory::PgStoreFactory;
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::sql_types;
use url::Url;
use uuid::Uuid;

use super::gateway_support::registration;
use super::support::{Database, at, hash};

#[derive(QueryableByName)]
struct Backend {
    #[diesel(sql_type = sql_types::Integer)]
    pid: i32,
}

#[derive(QueryableByName)]
struct Blocking {
    #[diesel(sql_type = sql_types::Bool)]
    blocked: bool,
}

fn wait_for_reader(observer: &mut PgConnection, writer_pid: i32, reader_pid: i32) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let blocking = diesel::sql_query("SELECT $1 = ANY(pg_blocking_pids($2)) AS blocked")
            .bind::<sql_types::Integer, _>(writer_pid)
            .bind::<sql_types::Integer, _>(reader_pid)
            .get_result::<Blocking>(observer)
            .unwrap();
        if blocking.blocked {
            return;
        }
        if Instant::now() >= deadline {
            panic!("reader did not reach the report transaction lock");
        }
        thread::yield_now();
    }
}

fn read(store: &dyn Store, workspace_id: Uuid, full: bool) -> MachineSnapshot {
    if full {
        store.machine_snapshots(workspace_id).unwrap().remove(0)
    } else {
        MachineSnapshot {
            machine: store.machines(workspace_id).unwrap().remove(0),
            report: None,
        }
    }
}

#[test]
fn machine_reads_keep_one_generation_when_a_report_commits_between_queries() {
    for (full, table) in [(false, "machine_disks"), (true, "machine_reports")] {
        let mut database = Database::new();
        let workspace = database.workspace();
        let token = hash();
        database
            .store
            .issue_install_token(workspace.id, Uuid::new_v4(), &token, at())
            .unwrap();
        let mut registration = registration(workspace.id, token);
        registration.contact.report.hardware.disks = vec![DiskReport {
            name: "/ci".into(),
            capacity_bytes: 1000,
            free_bytes: 111,
        }];
        assert!(database.store.register_machine(&registration).unwrap());
        let mut newer = registration.contact.report.clone();
        newer.hostname = "builder-new".into();
        newer.hardware.disks[0].free_bytes = 222;
        let tag = format!("machine-reader-{}", Uuid::new_v4());
        let mut url = Url::parse(&database.url).unwrap();
        url.query_pairs_mut()
            .append_pair("application_name", &tag)
            .append_pair("options", "-cstatement_timeout=20000");
        let reader = PgStoreFactory.connect(url.as_str()).unwrap();
        let mut observer = PgConnection::establish(&database.url).unwrap();
        let reader_pid = diesel::sql_query("SELECT pid FROM pg_stat_activity WHERE datname = current_database() AND application_name = $1")
            .bind::<sql_types::Text, _>(&tag).get_result::<Backend>(&mut observer).unwrap().pid;
        let writer_pid = diesel::sql_query("SELECT pg_backend_pid() AS pid")
            .get_result::<Backend>(&mut database.connection)
            .unwrap()
            .pid;
        database.connection.batch_execute("BEGIN").unwrap();
        diesel::update(machines::table.find(registration.machine_id))
            .set((
                machines::hostname.eq("builder-new"),
                machines::last_contact_at.eq(at() + TimeDelta::seconds(10)),
            ))
            .execute(&mut database.connection)
            .unwrap();
        diesel::update(
            machine_disks::table.filter(machine_disks::machine_id.eq(registration.machine_id)),
        )
        .set(machine_disks::free_bytes.eq(222))
        .execute(&mut database.connection)
        .unwrap();
        diesel::update(
            machine_reports::table.filter(machine_reports::machine_id.eq(registration.machine_id)),
        )
        .set(machine_reports::report.eq(serde_json::to_value(&newer).unwrap()))
        .execute(&mut database.connection)
        .unwrap();
        database
            .connection
            .batch_execute(&format!("LOCK TABLE {table} IN ACCESS EXCLUSIVE MODE"))
            .unwrap();
        let workspace_id = workspace.id;
        let pending = thread::spawn(move || read(&*reader, workspace_id, full));
        wait_for_reader(&mut observer, writer_pid, reader_pid);
        database.connection.batch_execute("COMMIT").unwrap();
        let snapshot = pending.join().unwrap();
        assert_eq!(snapshot.machine.hostname, "builder");
        assert_eq!(snapshot.machine.last_contact_at, Some(at()));
        assert_eq!(snapshot.machine.disks[0].free_bytes, 111);
        let expected = full.then_some(registration.contact.report);
        assert_eq!(snapshot.report, expected);
        let after = read(&*database.store, workspace_id, full);
        assert_eq!(after.machine.hostname, "builder-new");
        assert_eq!(
            after.machine.last_contact_at,
            Some(at() + TimeDelta::seconds(10))
        );
        assert_eq!(after.machine.disks[0].free_bytes, 222);
        assert_eq!(after.report, full.then_some(newer));
    }
}
