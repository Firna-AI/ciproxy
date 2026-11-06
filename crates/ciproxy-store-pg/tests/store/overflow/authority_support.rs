//! Liveness withdrawals exercise the existing startup and atomic-dispatch boundary.

use chrono::{DateTime, Utc};
use ciproxy_schema::labels::labels;
use ciproxy_schema::scale_sets::scale_sets;
use ciproxy_store_interface::overflow::admission::{
    PreparedDispatch, ReservationBudget, ReservationOutcome,
};
use ciproxy_store_interface::overflow::record::{StartPhase, StartRecord};
use ciproxy_store_interface::overflow::store::OverflowSession;
use diesel::prelude::*;
use rust_decimal::Decimal;

use super::super::support::{Database, at};

pub(super) fn assert_label_and_binding_liveness(
    database: &mut Database,
    session: &mut dyn OverflowSession,
    record: &StartRecord,
) {
    let mut pending = record.clone();
    let parameters = pending.dispatch.take().unwrap();
    pending.phase = StartPhase::Unsent;
    session.save_start(&pending).unwrap();
    let dispatch = PreparedDispatch {
        parameters,
        starts_at: pending.starts_at,
        budget: ReservationBudget {
            current_month_usd: Decimal::new(462, 3),
            next_month_usd: None,
        },
    };
    for retired_label in [true, false] {
        if retired_label {
            diesel::update(labels::table.find(record.label_id))
                .set(labels::deleted_at.eq(Some(at())))
                .execute(&mut database.connection)
                .unwrap();
        } else {
            diesel::update(scale_sets::table.find(record.scale_set_id))
                .set(scale_sets::remote_removed_at.eq(Some(at())))
                .execute(&mut database.connection)
                .unwrap();
        }
        assert_eq!(
            session.startup_account(record.delivery_id).unwrap(),
            None,
            "retired label: {retired_label}"
        );
        assert_eq!(
            session.begin_dispatch(&pending, &dispatch).unwrap(),
            ReservationOutcome::NotPermitted,
            "retired label: {retired_label}"
        );
        assert_eq!(session.snapshot().unwrap().starts, [pending.clone()]);
        if retired_label {
            diesel::update(labels::table.find(record.label_id))
                .set(labels::deleted_at.eq(None::<DateTime<Utc>>))
                .execute(&mut database.connection)
                .unwrap();
        } else {
            diesel::update(scale_sets::table.find(record.scale_set_id))
                .set(scale_sets::remote_removed_at.eq(None::<DateTime<Utc>>))
                .execute(&mut database.connection)
                .unwrap();
        }
    }
    session.save_start(record).unwrap();
}
