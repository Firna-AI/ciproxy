//! Typed database error translation without diagnostic value disclosure.

use ciproxy_store_interface::error::{Error, Result};
use diesel::result::{DatabaseErrorKind, QueryResult};

#[derive(Debug, thiserror::Error)]
pub(crate) enum InternalCause {
    #[error("[store-pg/connection] connection mutex is poisoned")]
    ConnectionPoisoned,
    #[error("[store-pg/row] stored row has an invalid shape")]
    InvalidRow,
}

pub(crate) trait QueryResultExt<T> {
    #[track_caller]
    fn store(self) -> Result<T>;
}

impl<T> QueryResultExt<T> for QueryResult<T> {
    #[track_caller]
    fn store(self) -> Result<T> {
        match self {
            Ok(value) => Ok(value),
            Err(source) => Err(query_error(source)),
        }
    }
}

#[track_caller]
pub(crate) fn query_error(source: diesel::result::Error) -> Error {
    match source {
        diesel::result::Error::NotFound => Error::NotFound,
        diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
            Error::Conflict
        }
        diesel::result::Error::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, _) => {
            Error::InvalidReference
        }
        diesel::result::Error::DatabaseError(
            DatabaseErrorKind::CheckViolation | DatabaseErrorKind::NotNullViolation,
            _,
        ) => Error::InvalidValue,
        source => Error::internal(source),
    }
}

pub(crate) fn require_record(count: usize) -> Result<()> {
    if count == 0 {
        return Err(Error::NotFound);
    }
    Ok(())
}
