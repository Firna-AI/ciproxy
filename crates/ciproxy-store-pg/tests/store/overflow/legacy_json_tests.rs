//! Reads of saved overflow JSON from before the provider field.

use ciproxy_store_interface::overflow::record::{EncryptedLaunch, StartPhase};
use ciproxy_store_interface::secrets::TokenHash;
use diesel::prelude::*;

use super::super::support::Database;
use super::support::{create_runner_intent, setup};

#[test]
fn reads_legacy_overflow_json_as_boat() {
    let mut database = Database::new();
    let (workspace, account, mut start) = setup(&database);
    create_runner_intent(&database, workspace, &start);
    start.phase = StartPhase::Unsent;
    start.dispatch = None;
    {
        let mut session = database.store.lock_overflow(workspace).unwrap();
        assert!(
            session
                .save_pending(&start, &EncryptedLaunch(vec![3]), &TokenHash::new(vec![4]))
                .unwrap()
        );
    }
    let intents = database.store.overflow_runner_intents(workspace).unwrap();
    assert_eq!(intents.len(), 1);
    for update in [
        "UPDATE overflow_accounts SET state_json = (state_json::jsonb - 'provider')::text
         WHERE state_json::jsonb ? 'provider'",
        "UPDATE overflow_starts SET record_json = (record_json::jsonb - 'provider')::text
         WHERE record_json::jsonb ? 'provider'",
        "UPDATE overflow_runner_intents SET body = body - 'provider' WHERE body ? 'provider'",
    ] {
        assert_eq!(
            diesel::sql_query(update)
                .execute(&mut database.connection)
                .unwrap(),
            1,
            "{update}"
        );
    }

    let snapshot = database.store.overflow_runtime_snapshot(workspace).unwrap();
    assert_eq!(snapshot.accounts, vec![account]);
    assert_eq!(snapshot.starts, vec![start]);
    assert_eq!(
        database.store.overflow_runner_intents(workspace).unwrap(),
        intents
    );
}
