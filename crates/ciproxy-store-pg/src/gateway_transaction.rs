//! Diesel-required transaction conversion that preserves the interface error.

use ciproxy_store_interface::error::{Error, Result};
use diesel::prelude::*;

use crate::error::query_error;

struct TransactionFailure(Error);

impl From<diesel::result::Error> for TransactionFailure {
    fn from(source: diesel::result::Error) -> Self {
        Self(query_error(source))
    }
}

pub(crate) fn transaction<T>(
    connection: &mut PgConnection,
    operation: impl FnOnce(&mut PgConnection) -> Result<T>,
) -> Result<T> {
    let result = connection.transaction(|connection| match operation(connection) {
        Ok(value) => Ok(value),
        Err(error) => Err(TransactionFailure(error)),
    });
    match result {
        Ok(value) => Ok(value),
        Err(TransactionFailure(error)) => Err(error),
    }
}
