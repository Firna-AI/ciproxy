//! Typed guest failures with a redacted internal-error display.

use internal_error::InternalError;

/// Failure at the guest lifecycle boundary.
#[derive(Debug, thiserror::Error, internal_error::ErrorContract)]
pub enum Error {
    /// No private runner input was delivered to this Boat sandbox.
    #[error("[ciproxy-guest] registration file is missing")]
    MissingRegistrationFile,
    /// The runner registration file can be read by another user.
    #[error("[ciproxy-guest] registration file must be private")]
    InsecureRegistrationFile,
    /// A required Linux command ended unsuccessfully.
    #[error("[ciproxy-guest] poweroff command failed")]
    PoweroffFailed,
    /// The cause stays available to trusted callers through `Debug`.
    #[error("[ciproxy-guest] internal error")]
    Internal(#[from] InternalError),
}

/// Result of a guest operation.
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// The context and the operating-system cause of an internal error, for
    /// the guest log. Other causes, such as JSON errors, can quote their
    /// input, which can be a registration, so only their kind is shown.
    pub fn diagnosis(&self) -> String {
        let Self::Internal(internal) = self else {
            return String::new();
        };
        let mut parts: Vec<String> = internal
            .context_frames()
            .iter()
            .rev()
            .map(|frame| frame.message().to_owned())
            .collect();
        for source in internal.source_chain() {
            match source.downcast_ref::<std::io::Error>() {
                Some(error) if error.raw_os_error().is_some() => parts.push(error.to_string()),
                Some(error) => parts.push(format!("I/O error of kind {:?}", error.kind())),
                None => parts.push("cause withheld".to_owned()),
            }
        }
        parts.join(": ")
    }
}

#[cfg(test)]
#[path = "_tests_/error_tests.rs"]
mod error_tests;
