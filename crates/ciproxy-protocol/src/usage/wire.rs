//! Decode optional usage without rejecting the surrounding message.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::value::RawValue;
use serde_json::{Map, Value};

use crate::message_usage::split_raw_field;

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

    /// Decode a usage object with its step text kept separate from its points.
    pub(crate) fn from_raw(raw: &RawValue) -> Self {
        let Ok((points, steps)) = split_raw_field(raw, "steps") else {
            return Self::PointsDropped;
        };
        Self::from_parts(points, || serde_json::from_str(steps?.get()).ok())
    }

    /// Keep a null exit field equivalent to an omitted field.
    pub(crate) fn optional_from_raw(raw: &RawValue) -> Option<Self> {
        (raw.get() != "null").then(|| Self::from_raw(raw))
    }

    fn from_parts(
        mut points: Map<String, Value>,
        decode_steps: impl FnOnce() -> Option<Vec<JobStep>>,
    ) -> Self {
        points.insert("steps".into(), Value::Array(Vec::new()));
        let Ok(mut usage) = serde_json::from_value::<JobUsage>(Value::Object(points)) else {
            return Self::PointsDropped;
        };
        match decode_steps() {
            Some(steps) => {
                usage.steps = steps;
                Self::Decoded(usage)
            }
            None => Self::StepsDropped(usage),
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
        let Ok(Value::Object(mut points)) = Value::deserialize(deserializer) else {
            return Ok(Self::PointsDropped);
        };
        let steps = points.remove("steps");
        Ok(Self::from_parts(points, || {
            steps.and_then(|value| serde_json::from_value(value).ok())
        }))
    }
}
