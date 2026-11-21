//! Preserve typed store failures while Diesel rolls back the whole mutation.

use ciproxy_store_interface::error::Result;
use diesel::prelude::*;

use crate::error::QueryResultExt;

pub(super) fn run<T>(
    connection: &mut PgConnection,
    action: impl FnOnce(&mut PgConnection) -> Result<T>,
) -> Result<T> {
    let mut failure = None;
    let result = connection.transaction::<T, diesel::result::Error, _>(|connection| {
        match action(connection) {
            Ok(value) => Ok(value),
            Err(error) => {
                failure = Some(error);
                Err(diesel::result::Error::RollbackTransaction)
            }
        }
    });
    if let Some(error) = failure {
        return Err(error);
    }
    result.store()
}
