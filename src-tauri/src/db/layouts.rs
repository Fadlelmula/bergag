use super::models::LayoutRecordDto;
use crate::error::{AppError, AppResult};
use crate::validation::{normalize_display_name, validate_layout_json};
use chrono::Utc;
use rusqlite::{params, Connection, Row};
use uuid::Uuid;

fn map_layout(row: &Row<'_>) -> rusqlite::Result<LayoutRecordDto> {
    let is_active: i64 = row.get("is_active")?;
    Ok(LayoutRecordDto {
        id: row.get("id")?,
        name: row.get("name")?,
        is_active: is_active == 1,
        layout_json: row.get("layout_json")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

fn get_layout(connection: &Connection, id: &str) -> AppResult<LayoutRecordDto> {
    connection
        .query_row(
            "SELECT id, name, is_active, layout_json, created_at, updated_at FROM layouts WHERE id = ?1",
            [id],
            map_layout,
        )
        .map_err(AppError::from)
}

pub fn list_layouts(connection: &Connection, profile_id: &str) -> AppResult<Vec<LayoutRecordDto>> {
    let mut statement = connection.prepare(
        "SELECT id, name, is_active, layout_json, created_at, updated_at
         FROM layouts WHERE profile_id = ?1 ORDER BY created_at ASC, name ASC",
    )?;
    let layouts = statement
        .query_map([profile_id], map_layout)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(AppError::from);
    layouts
}

pub fn get_active_layout(
    connection: &Connection,
    profile_id: &str,
) -> AppResult<Option<LayoutRecordDto>> {
    let mut statement = connection.prepare(
        "SELECT id, name, is_active, layout_json, created_at, updated_at
         FROM layouts WHERE profile_id = ?1 AND is_active = 1 LIMIT 1",
    )?;
    let mut rows = statement.query([profile_id])?;
    match rows.next()? {
        Some(row) => Ok(Some(map_layout(row)?)),
        None => Ok(None),
    }
}

pub fn save_layout(
    connection: &mut Connection,
    profile_id: &str,
    name: &str,
    layout_json: &str,
    activate: bool,
) -> AppResult<LayoutRecordDto> {
    let (display_name, normalized) = normalize_display_name(name, "Layout name")?;
    validate_layout_json(layout_json)?;
    let now = Utc::now().to_rfc3339();
    let transaction = connection.transaction()?;

    let existing_id: Option<String> = transaction
        .query_row(
            "SELECT id FROM layouts WHERE profile_id = ?1 AND name_normalized = ?2",
            params![profile_id, normalized],
            |row| row.get(0),
        )
        .optional()?;

    let id = if let Some(existing_id) = existing_id {
        transaction.execute(
            "UPDATE layouts SET name = ?1, layout_json = ?2, updated_at = ?3 WHERE id = ?4",
            params![display_name, layout_json, now, existing_id],
        )?;
        existing_id
    } else {
        let id = Uuid::new_v4().to_string();
        transaction.execute(
            "INSERT INTO layouts (id, profile_id, name, name_normalized, layout_json, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
            params![id, profile_id, display_name, normalized, layout_json, now],
        )?;
        id
    };

    let active_count: i64 = transaction.query_row(
        "SELECT COUNT(*) FROM layouts WHERE profile_id = ?1 AND is_active = 1",
        [profile_id],
        |row| row.get(0),
    )?;
    if activate || active_count == 0 {
        transaction.execute(
            "UPDATE layouts SET is_active = 0 WHERE profile_id = ?1",
            [profile_id],
        )?;
        transaction.execute(
            "UPDATE layouts SET is_active = 1, updated_at = ?2 WHERE id = ?1",
            params![id, now],
        )?;
    }

    let layout = get_layout(&transaction, &id)?;
    transaction.commit()?;
    Ok(layout)
}

pub fn activate_layout(
    connection: &mut Connection,
    profile_id: &str,
    id: &str,
) -> AppResult<LayoutRecordDto> {
    let transaction = connection.transaction()?;
    let belongs: bool = transaction.query_row(
        "SELECT EXISTS(SELECT 1 FROM layouts WHERE id = ?1 AND profile_id = ?2)",
        params![id, profile_id],
        |row| row.get(0),
    )?;
    if !belongs {
        return Err(AppError::NotFound(
            "That layout no longer exists.".to_string(),
        ));
    }
    transaction.execute(
        "UPDATE layouts SET is_active = 0 WHERE profile_id = ?1",
        [profile_id],
    )?;
    transaction.execute(
        "UPDATE layouts SET is_active = 1, updated_at = ?2 WHERE id = ?1",
        params![id, Utc::now().to_rfc3339()],
    )?;
    let layout = get_layout(&transaction, id)?;
    transaction.commit()?;
    Ok(layout)
}

pub fn delete_layout(
    connection: &mut Connection,
    profile_id: &str,
    id: &str,
) -> AppResult<Vec<LayoutRecordDto>> {
    let transaction = connection.transaction()?;
    let target = get_layout(&transaction, id)?;
    let belongs: bool = transaction.query_row(
        "SELECT EXISTS(SELECT 1 FROM layouts WHERE id = ?1 AND profile_id = ?2)",
        params![id, profile_id],
        |row| row.get(0),
    )?;
    if !belongs {
        return Err(AppError::NotFound(
            "That layout no longer exists.".to_string(),
        ));
    }
    if !target.is_active || list_layouts(&transaction, profile_id)?.len() <= 1 {
        transaction.execute(
            "DELETE FROM layouts WHERE id = ?1 AND profile_id = ?2",
            params![id, profile_id],
        )?;
    } else {
        transaction.execute(
            "DELETE FROM layouts WHERE id = ?1 AND profile_id = ?2",
            params![id, profile_id],
        )?;
        let replacement: Option<String> = transaction
            .query_row(
                "SELECT id FROM layouts WHERE profile_id = ?1 ORDER BY created_at ASC LIMIT 1",
                [profile_id],
                |row| row.get(0),
            )
            .optional()?;
        if let Some(replacement) = replacement {
            transaction.execute(
                "UPDATE layouts SET is_active = 1 WHERE id = ?1",
                [replacement],
            )?;
        }
    }
    let layouts = list_layouts(&transaction, profile_id)?;
    transaction.commit()?;
    Ok(layouts)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn test_connection() -> (Connection, String) {
        let mut connection = Connection::open_in_memory().unwrap();
        db::configure(&connection).unwrap();
        db::migrations::run(&mut connection).unwrap();
        db::profiles::ensure_default_profile(&mut connection).unwrap();
        let profile_id = db::profiles::get_active_profile(&connection).unwrap().id;
        (connection, profile_id)
    }

    #[test]
    fn saves_and_activates_layouts() {
        let (mut connection, profile_id) = test_connection();
        let default = save_layout(
            &mut connection,
            &profile_id,
            "Default",
            r#"{"version":1,"panels":{"sidebar":{"visible":true}}}"#,
            true,
        )
        .unwrap();
        assert!(default.is_active);

        let focus = save_layout(
            &mut connection,
            &profile_id,
            "Focus",
            r#"{"version":1,"panels":{"sidebar":{"visible":false}}}"#,
            true,
        )
        .unwrap();
        assert!(focus.is_active);
        assert!(!get_layout(&connection, &default.id).unwrap().is_active);

        let active = get_active_layout(&connection, &profile_id)
            .unwrap()
            .unwrap();
        assert_eq!(active.id, focus.id);

        let remaining = delete_layout(&mut connection, &profile_id, &focus.id).unwrap();
        assert_eq!(remaining.len(), 1);
        assert!(remaining[0].is_active);
    }

    #[test]
    fn rejects_invalid_layout_json() {
        let (mut connection, profile_id) = test_connection();
        let error = save_layout(
            &mut connection,
            &profile_id,
            "Broken",
            r#"{"version":1}"#,
            true,
        )
        .unwrap_err();
        assert!(matches!(error, AppError::Validation(_)));
    }
}
