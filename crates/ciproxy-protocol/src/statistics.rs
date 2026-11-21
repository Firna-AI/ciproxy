//! Independent measured or unavailable statistics with legacy report decoding.

use serde::{Deserialize, Deserializer, Serialize};

/// One statistics section, with no figures when its producer could not measure it.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Statistics<T> {
    /// Figures measured by their producer, including a true zero.
    Measured(T),
    /// The producer could not measure this section.
    Unavailable,
}

impl<T> Statistics<T> {
    /// Borrow measured figures while preserving their availability.
    pub fn as_ref(&self) -> Statistics<&T> {
        match self {
            Self::Measured(value) => Statistics::Measured(value),
            Self::Unavailable => Statistics::Unavailable,
        }
    }

    /// Borrow measured figures mutably while preserving their availability.
    pub fn as_mut(&mut self) -> Statistics<&mut T> {
        match self {
            Self::Measured(value) => Statistics::Measured(value),
            Self::Unavailable => Statistics::Unavailable,
        }
    }

    /// Map measured figures without creating figures for an unavailable section.
    pub fn map<U>(self, map: impl FnOnce(T) -> U) -> Statistics<U> {
        match self {
            Self::Measured(value) => Statistics::Measured(map(value)),
            Self::Unavailable => Statistics::Unavailable,
        }
    }

    /// Return figures only when their producer measured them.
    pub fn measured(self) -> Option<T> {
        match self {
            Self::Measured(value) => Some(value),
            Self::Unavailable => None,
        }
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Statistics<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let input = Input::<T>::deserialize(deserializer)?;
        match (input.status, input.figures) {
            (Status::Unavailable, _) => Ok(Self::Unavailable),
            (Status::Measured, Some(value)) => Ok(Self::Measured(value)),
            (Status::Measured, None) => Err(serde::de::Error::custom("missing measured figures")),
        }
    }
}

#[derive(Deserialize)]
struct Input<T> {
    #[serde(default)]
    status: Status,
    #[serde(flatten)]
    figures: Option<T>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Status {
    #[default]
    Measured,
    Unavailable,
}

#[cfg(test)]
#[path = "_tests_/statistics_tests.rs"]
mod statistics_tests;
