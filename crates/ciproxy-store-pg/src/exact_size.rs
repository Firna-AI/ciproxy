//! Checked resource integer conversion for Diesel's signed column types.

use std::error::Error;

use diesel::QueryResult;

pub(crate) fn convert<T, U>(value: U) -> QueryResult<T>
where
    T: TryFrom<U>,
    T::Error: Error + Send + Sync + 'static,
{
    match T::try_from(value) {
        Ok(value) => Ok(value),
        Err(error) => Err(diesel::result::Error::DeserializationError(Box::new(error))),
    }
}

pub(crate) fn json<T: serde::de::DeserializeOwned>(body: serde_json::Value) -> QueryResult<T> {
    match serde_json::from_value(body) {
        Ok(value) => Ok(value),
        Err(error) => Err(diesel::result::Error::DeserializationError(Box::new(error))),
    }
}

pub(crate) fn encode<T: serde::Serialize>(value: &T) -> QueryResult<serde_json::Value> {
    match serde_json::to_value(value) {
        Ok(value) => Ok(value),
        Err(error) => Err(diesel::result::Error::SerializationError(Box::new(error))),
    }
}
pub(crate) fn parse<T: std::str::FromStr>(value: &str) -> QueryResult<T>
where
    T::Err: Error + Send + Sync + 'static,
{
    match value.parse() {
        Ok(value) => Ok(value),
        Err(error) => Err(diesel::result::Error::DeserializationError(Box::new(error))),
    }
}
