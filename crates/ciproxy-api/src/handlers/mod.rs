//! HTTP entry points registered once with the OpenAPI router.

pub(crate) mod auth;
pub(crate) mod github_setup;
pub(crate) mod label_cleanup;
mod label_views;
pub(crate) mod labels;
mod machine_views;
pub(crate) mod machines;
pub(crate) mod members;
pub(crate) mod organizations;
pub(crate) mod sessions;
pub(crate) mod setup;
mod views;
