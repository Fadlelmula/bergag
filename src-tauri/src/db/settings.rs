use crate::error::{AppError, AppResult};
use crate::validation::{serialize_json_value, validate_setting_key};
use chrono::Utc;
use rusqlite::{params, Connection};
use serde_json::Value;

pub const ACTIVE_PROFILE_KEY: &str = "active_profile_id";

pub fn get_global_setting(connection: &Connection, key: &str) -> AppResult<Option<Value>> {
    validate_setting_key(key)?;
    let raw: Option<String> = connection
        .query_row(
            "SELECT value_json FROM app_settings WHERE key = ?1",
            [key],
            |row| row.get(0),
        )
        .optional()?;
    raw.map(|value| serde_json::from_str(&value).map_err(AppError::from))
        .transpose()
}

pub fn set_global_setting(connection: &Connection, key: &str, value: &Value) -> AppResult<()> {
    validate_setting_key(key)?;
    let serialized = serialize_json_value(value, 64_000, "setting value")?;
    connection.execute(
        "INSERT INTO app_settings (key, value_json, updated_at)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json,
           updated_at = excluded.updated_at",
        params![key, serialized, Utc::now().to_rfc3339()],
    )?;
    Ok(())
}

pub fn get_active_profile_id(connection: &Connection) -> AppResult<String> {
    let value = get_global_setting(connection, ACTIVE_PROFILE_KEY)?
        .and_then(|json| json.as_str().map(str::to_string))
        .ok_or_else(|| AppError::Internal("Active profile is not configured.".to_string()))?;
    Ok(value)
}

pub fn set_active_profile_id(connection: &Connection, profile_id: &str) -> AppResult<()> {
    set_global_setting(
        connection,
        ACTIVE_PROFILE_KEY,
        &Value::String(profile_id.to_string()),
    )
}

pub fn get_profile_setting(
    connection: &Connection,
    profile_id: &str,
    key: &str,
) -> AppResult<Option<Value>> {
    validate_setting_key(key)?;
    let raw: Option<String> = connection
        .query_row(
            "SELECT value_json FROM profile_settings WHERE profile_id = ?1 AND key = ?2",
            params![profile_id, key],
            |row| row.get(0),
        )
        .optional()?;
    raw.map(|value| serde_json::from_str(&value).map_err(AppError::from))
        .transpose()
}

pub fn set_profile_setting(
    connection: &Connection,
    profile_id: &str,
    key: &str,
    value: &Value,
) -> AppResult<()> {
    validate_setting_key(key)?;
    let serialized = serialize_json_value(value, 64_000, "setting value")?;
    connection.execute(
        "INSERT INTO profile_settings (profile_id, key, value_json, updated_at)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(profile_id, key) DO UPDATE SET value_json = excluded.value_json,
           updated_at = excluded.updated_at",
        params![profile_id, key, serialized, Utc::now().to_rfc3339()],
    )?;
    Ok(())
}

pub fn delete_profile_setting(
    connection: &Connection,
    profile_id: &str,
    key: &str,
) -> AppResult<()> {
    validate_setting_key(key)?;
    connection.execute(
        "DELETE FROM profile_settings WHERE profile_id = ?1 AND key = ?2",
        params![profile_id, key],
    )?;
    Ok(())
}

trait OptionalExtension<T> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error>;
}

impl<T> OptionalExtension<T> for Result<T, rusqlite::Error> {
    fn optional(self) -> Result<Option<T>, rusqlite::Error> {
        match self {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(error),
        }
    }
}
