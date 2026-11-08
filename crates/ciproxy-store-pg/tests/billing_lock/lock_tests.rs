//! Billing exclusion and release independent of backend disconnect timing.

use ciproxy_store_interface::store::StoreFactory;
use ciproxy_store_pg::factory::PgStoreFactory;

use super::database::Database;
use super::disconnect::HeldDisconnect;

#[test]
fn billing_session_releases_its_lock_before_the_server_disconnects() {
    let database = Database::new();
    let workspace = database.workspace();
    let proxy = HeldDisconnect::new(&database.url);
    let store = PgStoreFactory.connect(proxy.url()).unwrap();
    {
        let mut session = store.try_lock_billing(workspace.id).unwrap().unwrap();
        assert_eq!(session.record().unwrap().state.workspace_id, workspace.id);
        assert!(
            database
                .store
                .try_lock_billing(workspace.id)
                .unwrap()
                .is_none()
        );
    }
    proxy.wait_for_disconnect();
    let replacement = database.store.try_lock_billing(workspace.id).unwrap();
    assert!(
        replacement.is_some(),
        "closing the billing handle must release its lock before PostgreSQL disconnects"
    );
    assert_eq!(
        replacement.unwrap().record().unwrap().state.workspace_id,
        workspace.id
    );
}
