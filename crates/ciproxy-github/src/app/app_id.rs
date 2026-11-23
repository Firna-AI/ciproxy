//! GitHub App identity.

/// The configured GitHub App ID, distinct from the OAuth client ID.
#[derive(Clone, Debug)]
pub struct AppId(String);

impl AppId {
    /// Keep the configured ID unchanged.
    pub fn new(value: String) -> Self {
        Self(value)
    }

    /// Borrow the original App ID string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
