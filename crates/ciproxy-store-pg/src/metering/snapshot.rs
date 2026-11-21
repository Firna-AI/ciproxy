//! Shared transaction defaults for coherent metering snapshot queries.

use diesel::PgConnection;
use diesel::pg::TransactionBuilder;

/// Configure the read-only, repeatable-read transaction for a metering snapshot.
pub(super) fn transaction(connection: &mut PgConnection) -> TransactionBuilder<'_, PgConnection> {
    connection.build_transaction().repeatable_read().read_only()
}
