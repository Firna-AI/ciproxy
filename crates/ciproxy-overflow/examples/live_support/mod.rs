//! Credential-file reads and redacted failures shared by the live commands.

use std::{error, fmt, fs, path::Path};

use ciproxy_protocol::secret::Secret;

#[derive(thiserror::Error)]
#[error("[overflow_live] {stage} failed")]
pub(crate) struct Error {
    stage: &'static str,
    #[source]
    source: Box<dyn error::Error + Send + Sync>,
}

impl fmt::Debug for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

pub(crate) type Result<T> = std::result::Result<T, Error>;

pub(crate) trait SafeResult<T> {
    fn safe(self, stage: &'static str) -> Result<T>;
}

impl<T, E: Into<Box<dyn error::Error + Send + Sync>>> SafeResult<T> for std::result::Result<T, E> {
    fn safe(self, stage: &'static str) -> Result<T> {
        match self {
            Ok(value) => Ok(value),
            Err(source) => Err(Error {
                stage,
                source: source.into(),
            }),
        }
    }
}

pub(crate) fn secret(path: &Path) -> Result<Secret> {
    let bytes = fs::read_to_string(path).safe("read credential file")?;
    Ok(Secret::new(bytes.trim_end_matches(['\r', '\n'])))
}
