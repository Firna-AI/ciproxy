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

pub(super) struct Proration {
    pub(super) now: i64,
    pub(super) starts_at: i64,
    pub(super) ends_at: i64,
    pub(super) charges: Vec<u64>,
}

impl Proration {
    pub(super) fn amount(&self, request: &Request, date: &str) -> u64 {
        let at = request
            .value(date)
            .and_then(|at| at.parse().ok())
            .unwrap_or(self.now);
        ((self.ends_at - at) * 25_000 / (self.ends_at - self.starts_at)) as u64
    }
}
