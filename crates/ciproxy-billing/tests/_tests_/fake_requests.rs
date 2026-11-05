//! Captured HTTP inputs without authentication credentials.

#[derive(Clone, Debug)]
pub(super) struct Request {
    pub(super) method: String,
    pub(super) path: String,
    pub(super) form: Vec<(String, String)>,
    pub(super) key: Option<String>,
    pub(super) stripe_version: Option<String>,
}

impl Request {
    pub(super) fn value(&self, name: &str) -> Option<&str> {
        self.form
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }
}
