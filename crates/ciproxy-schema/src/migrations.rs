//! Embedded forward and reverse schema migrations.

use diesel_migrations::{EmbeddedMigrations, embed_migrations};

/// The ordered migrations applied by hosted PostgreSQL stores.
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
