//! Queries for exclusive overflow delivery, recovery, and key retention.

pub(crate) mod accounting;
mod accounts;
mod admission;
mod authority;
mod completion;
mod connections;
mod deliveries;
mod dispatch;
mod placements;
mod preparation;
mod queries;
mod rows;
mod starts;

use diesel::PgConnection;
use uuid::Uuid;

struct Session {
    connection: PgConnection,
    workspace_id: Uuid,
}
