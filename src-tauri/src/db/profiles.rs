use super::models::ProfileDto;
use super::settings::{
    get_active_profile_id, get_global_setting, set_active_profile_id, ACTIVE_PROFILE_KEY,
};
use crate::error::{AppError, AppResult};
use crate::validation::normalize_display_name;
use chrono::Utc;
use rusqlite::{params, Connection, Row};
use serde_json::Value;
use uuid::Uuid;

fn map_profile(row: &Row<'_>) -> rusqlite::Result<ProfileDto> {
    let is_locked: i64 = row.get("is_locked")?;
    Ok(ProfileDto {
        id: row.get("id")?,
        name: row.get("name")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        is_locked: is_locked == 1,
        lock_hint: row.get("lock_hint")?,
    })
}

pub fn list_profiles(connection: &Connection) -> AppResult<Vec<ProfileDto>> {
    let mut statement = connection.prepare(
        "SELECT id, name, created_at, updated_at, is_locked, lock_hint
         FROM profiles ORDER BY created_at ASC, name_normalized ASC",
    )?;
    let rows = statement.query_map([], map_profile)?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(AppError::from)
}

pub fn get_profile(connection: &Connection, id: &str) -> AppResult<ProfileDto> {
    connection
        .query_row(
            "SELECT id, name, created_at, updated_at, is_locked, lock_hint FROM profiles WHERE id = ?1",
            [id],
            map_profile,
        )
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => {
                AppError::NotFound("That profile no longer exists.".to_string())
            }
            other => AppError::from(other),
        })
}

pub fn get_active_profile(connection: &Connection) -> AppResult<ProfileDto> {
    let id = get_active_profile_id(connection)?;
    get_profile(connection, &id)
}

pub fn ensure_default_profile(connection: &mut Connection) -> AppResult<()> {
    let count: i64 = connection.query_row("SELECT COUNT(*) FROM profiles", [], |row| row.get(0))?;
    let transaction = connection.transaction()?;

    if count == 0 {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        transaction.execute(
            "INSERT INTO profiles (id, name, name_normalized, created_at, updated_at)
             VALUES (?1, 'Default', 'default', ?2, ?2)",
            params![id, now],
        )?;
        set_active_profile_id(&transaction, &id)?;
        transaction.commit()?;
        return Ok(());
    }

    let active_valid = match get_global_setting(&transaction, ACTIVE_PROFILE_KEY)? {
        Some(Value::String(id)) => {
            let exists: bool = transaction.query_row(
                "SELECT EXISTS(SELECT 1 FROM profiles WHERE id = ?1)",
                [&id],
                |row| row.get(0),
            )?;
            exists
        }
        _ => false,
    };

    if !active_valid {
        let first: String = transaction.query_row(
            "SELECT id FROM profiles ORDER BY created_at ASC, name_normalized ASC LIMIT 1",
            [],
            |row| row.get(0),
        )?;
        set_active_profile_id(&transaction, &first)?;
    }

    transaction.commit()?;
    Ok(())
}

pub fn create_profile(connection: &mut Connection, raw_name: &str) -> AppResult<ProfileDto> {
    let (display_name, normalized) = normalize_display_name(raw_name, "Profile name")?;
    let transaction = connection.transaction()?;
    let duplicate: bool = transaction.query_row(
        "SELECT EXISTS(SELECT 1 FROM profiles WHERE name_normalized = ?1)",
        [&normalized],
        |row| row.get(0),
    )?;
    if duplicate {
        return Err(AppError::Conflict(
            "A profile with that name already exists.".to_string(),
        ));
    }

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    transaction.execute(
        "INSERT INTO profiles (id, name, name_normalized, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?4)",
        params![id, display_name, normalized, now],
    )?;
    let profile = get_profile(&transaction, &id)?;
    transaction.commit()?;
    Ok(profile)
}

pub fn rename_profile(
    connection: &mut Connection,
    id: &str,
    raw_name: &str,
) -> AppResult<ProfileDto> {
    let (display_name, normalized) = normalize_display_name(raw_name, "Profile name")?;
    let transaction = connection.transaction()?;
    let duplicate: bool = transaction.query_row(
        "SELECT EXISTS(SELECT 1 FROM profiles WHERE name_normalized = ?1 AND id <> ?2)",
        params![normalized, id],
        |row| row.get(0),
    )?;
    if duplicate {
        return Err(AppError::Conflict(
            "A profile with that name already exists.".to_string(),
        ));
    }

    let updated = transaction.execute(
        "UPDATE profiles SET name = ?1, name_normalized = ?2, updated_at = ?3 WHERE id = ?4",
        params![display_name, normalized, Utc::now().to_rfc3339(), id],
    )?;
    if updated == 0 {
        return Err(AppError::NotFound(
            "That profile no longer exists.".to_string(),
        ));
    }
    let profile = get_profile(&transaction, id)?;
    transaction.commit()?;
    Ok(profile)
}

pub fn switch_profile(connection: &mut Connection, id: &str) -> AppResult<ProfileDto> {
    let profile = get_profile(connection, id)?;
    let transaction = connection.transaction()?;
    set_active_profile_id(&transaction, &profile.id)?;
    transaction.commit()?;
    Ok(profile)
}

pub fn delete_profile(connection: &mut Connection, id: &str) -> AppResult<Vec<ProfileDto>> {
    let transaction = connection.transaction()?;
    let count: i64 =
        transaction.query_row("SELECT COUNT(*) FROM profiles", [], |row| row.get(0))?;
    if count <= 1 {
        return Err(AppError::Conflict(
            "Bergag requires at least one profile.".to_string(),
        ));
    }
    let target = get_profile(&transaction, id)?;
    let active_id = get_active_profile_id(&transaction)?;

    let mut provider_keys =
        transaction.prepare("SELECT id FROM ai_providers WHERE profile_id = ?1")?;
    let provider_ids = provider_keys
        .query_map([&target.id], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, rusqlite::Error>>()?;
    drop(provider_keys);
    for provider_id in &provider_ids {
        crate::credentials::delete_api_key(&target.id, provider_id)?;
    }

    transaction.execute("DELETE FROM profiles WHERE id = ?1", [&target.id])?;

    if active_id == target.id {
        let replacement: String = transaction.query_row(
            "SELECT id FROM profiles ORDER BY created_at ASC, name_normalized ASC LIMIT 1",
            [],
            |row| row.get(0),
        )?;
        set_active_profile_id(&transaction, &replacement)?;
    }

    let profiles = list_profiles(&transaction)?;
    transaction.commit()?;
    Ok(profiles)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use crate::db::layouts;

    fn test_connection() -> Connection {
        let mut connection = Connection::open_in_memory().unwrap();
        db::configure(&connection).unwrap();
        db::migrations::run(&mut connection).unwrap();
        ensure_default_profile(&mut connection).unwrap();
        connection
    }

    #[test]
    fn creates_default_profile() {
        let connection = test_connection();
        let profiles = list_profiles(&connection).unwrap();
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].name, "Default");
        assert_eq!(get_active_profile(&connection).unwrap().id, profiles[0].id);
    }

    #[test]
    fn isolates_settings_and_layouts_between_profiles() {
        let mut connection = test_connection();
        let research = create_profile(&mut connection, "Research").unwrap();

        super::super::settings::set_profile_setting(
            &connection,
            &research.id,
            "reading.mode",
            &Value::String("focus".to_string()),
        )
        .unwrap();
        layouts::save_layout(
            &mut connection,
            &research.id,
            "Default",
            r#"{"version":1,"panels":{}}"#,
            true,
        )
        .unwrap();

        let default = get_active_profile(&connection).unwrap();
        assert_eq!(
            super::super::settings::get_profile_setting(&connection, &default.id, "reading.mode")
                .unwrap(),
            None
        );
        assert!(layouts::list_layouts(&connection, &default.id)
            .unwrap()
            .is_empty());
        assert_eq!(
            super::super::settings::get_profile_setting(&connection, &research.id, "reading.mode")
                .unwrap(),
            Some(Value::String("focus".to_string()))
        );
    }

    #[test]
    fn switching_and_deleting_profiles_updates_active_pointer() {
        let mut connection = test_connection();
        let default = get_active_profile(&connection).unwrap();
        let research = create_profile(&mut connection, "Research").unwrap();

        let active = switch_profile(&mut connection, &research.id).unwrap();
        assert_eq!(active.id, research.id);

        let remaining = delete_profile(&mut connection, &research.id).unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(get_active_profile(&connection).unwrap().id, default.id);
    }

    #[test]
    fn refuses_to_delete_last_profile() {
        let mut connection = test_connection();
        let default = get_active_profile(&connection).unwrap();
        let error = delete_profile(&mut connection, &default.id).unwrap_err();
        assert!(matches!(error, AppError::Conflict(_)));
    }

    #[test]
    fn rejects_duplicate_profile_names_case_insensitively() {
        let mut connection = test_connection();
        create_profile(&mut connection, "Research").unwrap();
        let error = create_profile(&mut connection, "  research  ").unwrap_err();
        assert!(matches!(error, AppError::Conflict(_)));
    }

    #[test]
    fn deleting_profile_cascades_its_data() {
        let mut connection = test_connection();
        let temporary = create_profile(&mut connection, "Temporary").unwrap();
        super::super::settings::set_profile_setting(
            &connection,
            &temporary.id,
            "appearance.theme",
            &Value::String("light".to_string()),
        )
        .unwrap();
        delete_profile(&mut connection, &temporary.id).unwrap();

        let count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM profile_settings WHERE profile_id = ?1",
                [&temporary.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }
}
