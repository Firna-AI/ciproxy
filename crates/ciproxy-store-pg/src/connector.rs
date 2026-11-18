//! One bounded libpq connection policy for queries, leases, and migrations.

use ciproxy_store_interface::error::{Result, ResultExt};
use diesel::connection::SimpleConnection;
use diesel::{Connection, PgConnection};

use crate::error::QueryResultExt;

/// The finite server limits required by a connection's work.
pub(crate) enum ConnectionUse {
    /// Ordinary reads, mutations, and dedicated advisory-lock sessions.
    Queries,
    /// Schema changes with a longer, still finite statement budget.
    Migrations,
}

/// Opens connections with transport protection and server-side limits.
pub(crate) trait Connector: Send + Sync {
    /// Applies all selected limits before returning the database session.
    fn connect(&self, usage: ConnectionUse) -> Result<PgConnection>;
}

/// Retains connection parameters privately, without diagnostic disclosure.
pub(crate) struct BoundedConnector {
    parameters: String,
}

impl BoundedConnector {
    /// Preserves libpq connection syntax while replacing transport defaults.
    pub(crate) fn new(database_url: &str) -> Self {
        let settings = [
            "connect_timeout=10",
            "keepalives=1",
            "keepalives_idle=10",
            "keepalives_interval=5",
            "keepalives_count=3",
            "tcp_user_timeout=30000",
        ];
        let parameters = if database_url.starts_with("postgresql://")
            || database_url.starts_with("postgres://")
        {
            let separator = if database_url.ends_with(['?', '&']) {
                ""
            } else if database_url.contains('?') {
                "&"
            } else {
                "?"
            };
            format!("{database_url}{separator}{}", settings.join("&"))
        } else {
            format!(
                "{} {}",
                terminated_keywords(database_url),
                settings.join(" ")
            )
        };
        Self { parameters }
    }
}

/// libpq ignores one final unpaired escape in an unquoted keyword value.
/// Remove that escape so it cannot consume the new parameter delimiter.
fn terminated_keywords(parameters: &str) -> &str {
    let escapes = parameters
        .chars()
        .rev()
        .take_while(|character| *character == '\\')
        .count();
    match parameters.strip_suffix('\\') {
        Some(prefix) if escapes % 2 == 1 => prefix,
        _ => parameters,
    }
}

impl Connector for BoundedConnector {
    fn connect(&self, usage: ConnectionUse) -> Result<PgConnection> {
        let mut connection = PgConnection::establish(&self.parameters).internal()?;
        let limits = match usage {
            ConnectionUse::Queries => {
                "SET statement_timeout = 30000;
                 SET lock_timeout = 5000;
                 SET idle_in_transaction_session_timeout = 30000;"
            }
            ConnectionUse::Migrations => {
                "SET statement_timeout = 600000;
                 SET lock_timeout = 30000;
                 SET idle_in_transaction_session_timeout = 60000;"
            }
        };
        connection.batch_execute(limits).store()?;
        Ok(connection)
    }
}
