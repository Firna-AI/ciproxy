//! Decode optional usage without rejecting the surrounding message.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use super::{JobStep, JobUsage};

/// Usage on a runner exit or sample report, including partial decode results.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UsageReport {
    /// Points and steps both decoded. They may still fail validation.
    Decoded(JobUsage),
    /// Only the points decoded. The step list is empty.
    StepsDropped(JobUsage),
    /// The points did not decode. None of the input is retained.
    PointsDropped,
}

/// A safe reason to log for a usage decode drop.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UsageDecodeDrop {
    /// The points could not be read, so the whole usage report was dropped.
    Points,
    /// The steps could not be read, but the points were kept.
    Steps,
}

impl fmt::Display for UsageDecodeDrop {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Points => "usage points could not be decoded",
            Self::Steps => "usage steps could not be decoded",
        })
    }
}

impl UsageReport {
    /// Return the usage that decoded and a reason for any part that did not.
    pub fn into_parts(self) -> (Option<JobUsage>, Option<UsageDecodeDrop>) {
        match self {
            Self::Decoded(usage) => (Some(usage), None),
            Self::StepsDropped(usage) => (Some(usage), Some(UsageDecodeDrop::Steps)),
            Self::PointsDropped => (None, Some(UsageDecodeDrop::Points)),
        }
    }
}

impl From<JobUsage> for UsageReport {
    fn from(usage: JobUsage) -> Self {
        Self::Decoded(usage)
    }
}

impl Serialize for UsageReport {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Decoded(usage) | Self::StepsDropped(usage) => usage.serialize(serializer),
            Self::PointsDropped => serializer.serialize_none(),
        }
    }
}

impl<'de> Deserialize<'de> for UsageReport {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let Ok(Value::Object(mut fields)) = Value::deserialize(deserializer) else {
            return Ok(Self::PointsDropped);
        };
        let steps = fields.insert("steps".into(), Value::Array(Vec::new()));
        let Ok(mut usage) = serde_json::from_value::<JobUsage>(Value::Object(fields)) else {
            return Ok(Self::PointsDropped);
        };
        match steps.and_then(|value| serde_json::from_value::<Vec<JobStep>>(value).ok()) {
            Some(steps) => {
                usage.steps = steps;
                Ok(Self::Decoded(usage))
            }
            None => Ok(Self::StepsDropped(usage)),
        }
    }
}
