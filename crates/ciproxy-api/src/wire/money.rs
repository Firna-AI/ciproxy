//! Exact integer cents with a decimal-string HTTP representation.

use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use utoipa::ToSchema;

/// Integer US cents encoded as a string to preserve JavaScript precision.
#[derive(Clone, Copy, Debug, ToSchema)]
#[schema(value_type = String, pattern = "^-?[0-9]+$")]
pub struct Cents(
    /// The exact integer number of US cents.
    pub i128,
);

impl Cents {
    /// Convert exact stored USD into cents without rounding or an i64 ceiling.
    pub fn from_usd(usd: Decimal) -> Option<Self> {
        let value = usd.mantissa();
        let scale = usd.scale();
        if scale <= 2 {
            return value.checked_mul(10i128.pow(2 - scale)).map(Self);
        }
        let divisor = 10i128.pow(scale - 2);
        (value % divisor == 0).then(|| Self(value / divisor))
    }

    /// Convert integer cents into the existing store's exact Decimal USD type.
    pub fn usd(self) -> Result<Decimal, rust_decimal::Error> {
        Decimal::try_from_i128_with_scale(self.0, 2)
    }
}

impl Serialize for Cents {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.to_string())
    }
}
impl<'de> Deserialize<'de> for Cents {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.parse::<i128>() {
            Ok(value) => Ok(Self(value)),
            Err(_) => Err(serde::de::Error::custom(
                "expected integer US cents as a decimal string",
            )),
        }
    }
}
