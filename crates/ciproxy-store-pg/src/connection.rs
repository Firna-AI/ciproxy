//! Private connection ownership shared by query implementations.

use std::sync::{Mutex, MutexGuard};

use ciproxy_store_interface::error::{Error, Result};
use ciproxy_store_interface::store::Store;
use diesel::PgConnection;

use crate::error::InternalCause;

pub(crate) struct PgStore {
    pub(crate) connection: Mutex<PgConnection>,
    pub(crate) database_url: String,
}

impl PgStore {
    pub(crate) fn connection(&self) -> Result<MutexGuard<'_, PgConnection>> {
        match self.connection.lock() {
            Ok(connection) => Ok(connection),
            Err(_) => Err(Error::internal(InternalCause::ConnectionPoisoned)),
        }
    }
}

impl Store for PgStore {}
