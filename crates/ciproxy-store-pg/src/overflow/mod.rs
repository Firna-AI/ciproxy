//! Queries for exclusive overflow delivery, recovery, and key retention.

pub(crate) mod accounting;
mod accounts;
mod authority;
mod completion;
mod connections;
mod deliveries;
mod placements;
mod queries;
mod rows;
mod starts;

use diesel::PgConnection;
use uuid::Uuid;

struct Session {
    connection: PgConnection,
    workspace_id: Uuid,
}
