use rusqlite::{types::Type, Error};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

pub fn new_id(prefix: &str) -> String {
    format!("{prefix}_{}", uuid::Uuid::new_v4().simple())
}

pub fn now() -> String {
    OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .expect("RFC3339 formatting should not fail")
}

pub fn encode_json<T: Serialize>(value: &T) -> rusqlite::Result<String> {
    serde_json::to_string(value).map_err(|error| Error::ToSqlConversionFailure(Box::new(error)))
}

pub fn decode_json<T: for<'de> Deserialize<'de>>(value: String, column_index: usize) -> rusqlite::Result<T> {
    serde_json::from_str(&value)
        .map_err(|error| Error::FromSqlConversionFailure(column_index, Type::Text, Box::new(error)))
}
