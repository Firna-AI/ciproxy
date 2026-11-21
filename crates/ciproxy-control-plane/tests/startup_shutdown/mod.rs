//! Isolated executable, CDN, PostgreSQL, and HTTP shutdown fixtures.

mod cdn;
pub(super) mod database;
pub(super) mod fixture;
pub(super) mod http;
pub(super) mod process;
mod secrets;
pub(super) mod sockets;
