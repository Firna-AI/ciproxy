//! A failed plan-price write cannot expose its billing state or event receipt.

use chrono::Duration;
use ciproxy_store_interface::billing::commands::{BillingCommand, CommandKind};
use ciproxy_store_interface::error::Error;
use diesel::prelude::*;
use uuid::Uuid;

use super::database::Database;
use super::history_support::{paid, plans};

#[test]
fn failed_plan_price_write_rolls_back_state_receipt_and_command_completion() {
    let database = Database::new();
    let workspace = database.workspace();
    let mut session = database
        .store
        .try_lock_billing(workspace.id)
        .unwrap()
        .unwrap();
    let initial = session.record().unwrap().state;
    let existing = plans(&database, &workspace);
    let command = BillingCommand {
        id: Uuid::new_v4(),
        started_at: initial.free_started_at,
        payment_expires_at: None,
        kind: CommandKind::Cancel,
    };
    session.save_command(&command).unwrap();
    let mut connection = PgConnection::establish(&database.url).unwrap();
    diesel::sql_query(
        "ALTER TABLE metering_plan_costs ADD CONSTRAINT reject_paid_price CHECK (monthly_usd = 0)",
    )
    .execute(&mut connection)
    .unwrap();
    let at = initial.free_started_at + Duration::days(1);
    let confirmed = paid(&initial, at);
    assert!(matches!(
        session.commit(Some(&confirmed), Some("evt_atomic_price"), true, false, at),
        Err(Error::InvalidValue)
    ));
    let record = session.record().unwrap();
    assert_eq!(record.state, initial);
    assert_eq!(record.command, Some(command));
    assert!(!session.event_applied("evt_atomic_price").unwrap());
    assert_eq!(plans(&database, &workspace), existing);
    diesel::sql_query("ALTER TABLE metering_plan_costs DROP CONSTRAINT reject_paid_price")
        .execute(&mut connection)
        .unwrap();
    session
        .commit(Some(&confirmed), Some("evt_atomic_price"), true, false, at)
        .unwrap();
    assert_eq!(session.record().unwrap().state, confirmed);
    assert_eq!(session.record().unwrap().command, None);
    assert!(session.event_applied("evt_atomic_price").unwrap());
    assert_eq!(plans(&database, &workspace).len(), 2);
}
