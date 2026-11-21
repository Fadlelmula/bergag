use super::models::{
    BackupIndex, BackupManifest, BackupRecord, BackupVerification, RestoreOutcome,
    AUTOMATIC_RETENTION, BACKUP_FORMAT, DATABASE_SUFFIX, MANIFEST_SUFFIX,
};
use crate::db;
use crate::error::{AppError, AppResult};
use crate::validation::normalize_display_name;
use chrono::Utc;
use rusqlite::{backup, OpenFlags, MAIN_DB};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use uuid::Uuid;

const MANIFEST_MAX_BYTES: usize = 16_384;

fn index_path(backup_dir: &Path) -> PathBuf {
    backup_dir.join("index.json")
}

fn database_path(backup_dir: &Path, id: &str) -> PathBuf {
    backup_dir.join(format!("{id}{DATABASE_SUFFIX}"))
}

fn manifest_path(backup_dir: &Path, id: &str) -> PathBuf {
    backup_dir.join(format!("{id}{MANIFEST_SUFFIX}"))
}

fn validate_id(id: &str) -> AppResult<()> {
    if Uuid::parse_str(id).is_err() {
        return Err(AppError::Validation(
            "That backup identifier is not valid.".to_string(),
        ));
    }
    Ok(())
}

pub fn initialize(backup_dir: &Path) -> AppResult<()> {
    fs::create_dir_all(backup_dir).map_err(|error| {
        AppError::Internal(format!("Could not create the backup directory: {error}"))
    })?;
    let path = index_path(backup_dir);
    if !path.exists() {
        write_index(backup_dir, &BackupIndex::default())?;
    }
    Ok(())
}

fn read_index(backup_dir: &Path) -> AppResult<BackupIndex> {
    let path = index_path(backup_dir);
    let contents = fs::read_to_string(&path)
        .map_err(|error| AppError::Internal(format!("Could not read the backup index: {error}")))?;
    if contents.len() > 1_048_576 {
        return Err(AppError::Internal(
            "The backup index is unexpectedly large.".to_string(),
        ));
    }
    serde_json::from_str(&contents).map_err(|error| {
        AppError::Internal(format!("The backup index could not be parsed: {error}"))
    })
}

fn write_index(backup_dir: &Path, index: &BackupIndex) -> AppResult<()> {
    let serialized = serde_json::to_vec_pretty(index).map_err(|error| {
        AppError::Internal(format!("Could not serialize the backup index: {error}"))
    })?;
    if serialized.len() > 1_048_576 {
        return Err(AppError::Internal(
            "The backup index would be too large.".to_string(),
        ));
    }
    let temporary = backup_dir.join("index.json.tmp");
    fs::write(&temporary, &serialized).map_err(|error| {
        AppError::Internal(format!("Could not write the backup index: {error}"))
    })?;
    let path = index_path(backup_dir);
    if path.exists() {
        fs::remove_file(&path).map_err(|error| {
            AppError::Internal(format!("Could not replace the backup index: {error}"))
        })?;
    }
    fs::rename(&temporary, &path).map_err(|error| {
        AppError::Internal(format!("Could not finalize the backup index: {error}"))
    })
}

fn hash_file(path: &Path) -> AppResult<String> {
    let mut file = File::open(path)
        .map_err(|error| AppError::Internal(format!("Could not read a backup file: {error}")))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(|error| {
            AppError::Internal(format!("Could not hash a backup file: {error}"))
        })?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

fn integrity_check(connection: &rusqlite::Connection) -> AppResult<()> {
    let result: String = connection.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
    if result != "ok" {
        return Err(AppError::Internal(
            "A local database integrity check failed.".to_string(),
        ));
    }
    Ok(())
}

fn schema_version(connection: &rusqlite::Connection) -> AppResult<u32> {
    let version: i64 = connection.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    u32::try_from(version).map_err(|_| {
        AppError::Internal("The local database schema version is out of range.".to_string())
    })
}

fn snapshot_metadata(database_file: &Path) -> AppResult<(u32, Vec<String>)> {
    let connection = rusqlite::Connection::open_with_flags(
        database_file,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    integrity_check(&connection)?;
    let version = schema_version(&connection)?;
    let profiles = db::profiles::list_profiles(&connection)?
        .into_iter()
        .map(|profile| profile.id)
        .collect();
    Ok((version, profiles))
}

fn read_manifest(path: &Path) -> AppResult<BackupManifest> {
    let contents = fs::read_to_string(path).map_err(|error| {
        AppError::Validation(format!("The backup manifest could not be read: {error}"))
    })?;
    if contents.len() > MANIFEST_MAX_BYTES {
        return Err(AppError::Validation(
            "The backup manifest is unexpectedly large.".to_string(),
        ));
    }
    let manifest: BackupManifest = serde_json::from_str(&contents).map_err(|error| {
        AppError::Validation(format!("The backup manifest is invalid: {error}"))
    })?;
    if manifest.format != BACKUP_FORMAT {
        return Err(AppError::Validation(
            "This backup format is not supported by this Bergag version.".to_string(),
        ));
    }
    Ok(manifest)
}

fn validate_snapshot(backup_dir: &Path, id: &str) -> AppResult<BackupManifest> {
    validate_id(id)?;
    let manifest = read_manifest(&manifest_path(backup_dir, id))?;
    let database_file = database_path(backup_dir, id);
    let expected_filename = database_file
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("backup.sqlite3");
    if manifest.database_filename != expected_filename {
        return Err(AppError::Validation(
            "The backup manifest does not match its database file.".to_string(),
        ));
    }
    let actual_hash = hash_file(&database_file)?;
    if actual_hash != manifest.database_sha256 {
        return Err(AppError::Validation(
            "The backup contents do not match their recorded checksum.".to_string(),
        ));
    }
    let (version, profile_ids) = snapshot_metadata(&database_file)?;
    if version != manifest.schema_version || profile_ids != manifest.profile_ids {
        return Err(AppError::Validation(
            "The backup contents do not match their manifest.".to_string(),
        ));
    }
    Ok(manifest)
}

fn prune_automatic(backup_dir: &Path, index: &mut BackupIndex) -> AppResult<()> {
    let mut automatic: Vec<(usize, &BackupRecord)> = index
        .records
        .iter()
        .enumerate()
        .filter(|(_, record)| record.automatic)
        .collect();
    automatic.sort_by(|left, right| right.1.created_at.cmp(&left.1.created_at));
    let remove = automatic
        .into_iter()
        .skip(AUTOMATIC_RETENTION)
        .map(|(position, _)| position)
        .collect::<Vec<_>>();
    for position in remove.into_iter().rev() {
        let record = &index.records[position];
        validate_id(&record.id)?;
        let record = index.records.remove(position);
        let _ = fs::remove_file(database_path(backup_dir, &record.id));
        let _ = fs::remove_file(manifest_path(backup_dir, &record.id));
    }
    Ok(())
}

pub fn create(
    backup_dir: &Path,
    connection: &rusqlite::Connection,
    label: Option<&str>,
    automatic: bool,
) -> AppResult<BackupRecord> {
    initialize(backup_dir)?;
    let (label, _) = normalize_display_name(label.unwrap_or("Manual backup"), "Backup label")?;
    let id = Uuid::new_v4().to_string();
    let created_at = Utc::now().to_rfc3339();
    let database_file = database_path(backup_dir, &id);
    let temporary_database = backup_dir.join(format!("{id}.tmp{DATABASE_SUFFIX}"));
    let manifest_file = manifest_path(backup_dir, &id);
    let temporary_manifest = backup_dir.join(format!("{id}.tmp{MANIFEST_SUFFIX}"));

    let cleanup = || {
        let _ = fs::remove_file(&temporary_database);
        let _ = fs::remove_file(&temporary_manifest);
    };

    connection
        .backup(MAIN_DB, &temporary_database, None::<fn(backup::Progress)>)
        .map_err(|error| {
            AppError::Internal(format!("Could not create a database backup: {error}"))
        })?;

    let (schema_version, profile_ids) = match snapshot_metadata(&temporary_database) {
        Ok(metadata) => metadata,
        Err(error) => {
            cleanup();
            return Err(error);
        }
    };
    let database_sha256 = match hash_file(&temporary_database) {
        Ok(hash) => hash,
        Err(error) => {
            cleanup();
            return Err(error);
        }
    };
    let manifest = BackupManifest {
        format: BACKUP_FORMAT.to_string(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        schema_version,
        created_at: created_at.clone(),
        profile_count: profile_ids.len(),
        profile_ids,
        database_sha256,
        database_filename: database_file
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("backup.sqlite3")
            .to_string(),
    };
    let serialized = match serde_json::to_vec_pretty(&manifest) {
        Ok(value) if value.len() <= MANIFEST_MAX_BYTES => value,
        Ok(_) => {
            cleanup();
            return Err(AppError::Internal(
                "The backup manifest would be too large.".to_string(),
            ));
        }
        Err(error) => {
            cleanup();
            return Err(AppError::Internal(format!(
                "Could not serialize the backup manifest: {error}"
            )));
        }
    };
    if fs::write(&temporary_manifest, &serialized).is_err()
        || fs::rename(&temporary_database, &database_file).is_err()
        || fs::rename(&temporary_manifest, &manifest_file).is_err()
    {
        cleanup();
        let _ = fs::remove_file(&database_file);
        let _ = fs::remove_file(&manifest_file);
        return Err(AppError::Internal(
            "Could not finalize the backup files.".to_string(),
        ));
    }

    let size_bytes = match fs::metadata(&database_file) {
        Ok(metadata) => metadata.len(),
        Err(error) => {
            cleanup();
            return Err(AppError::Internal(format!(
                "Could not measure the backup file: {error}"
            )));
        }
    };
    let record = BackupRecord {
        id: id.clone(),
        label,
        created_at,
        schema_version,
        profile_count: manifest.profile_count,
        size_bytes,
        automatic,
    };
    let mut index = read_index(backup_dir)?;
    index.records.push(record.clone());
    prune_automatic(backup_dir, &mut index)?;
    write_index(backup_dir, &index)?;
    Ok(record)
}

pub fn list(backup_dir: &Path) -> AppResult<Vec<BackupRecord>> {
    initialize(backup_dir)?;
    let mut records = read_index(backup_dir)?.records;
    records.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    Ok(records)
}

pub fn verify(backup_dir: &Path, id: &str) -> AppResult<BackupVerification> {
    match validate_snapshot(backup_dir, id) {
        Ok(_) => Ok(BackupVerification {
            valid: true,
            reason: None,
        }),
        Err(AppError::Validation(reason)) => Ok(BackupVerification {
            valid: false,
            reason: Some(reason),
        }),
        Err(error) => Err(error),
    }
}

pub fn delete(backup_dir: &Path, id: &str) -> AppResult<()> {
    validate_id(id)?;
    let mut index = read_index(backup_dir)?;
    let position = index
        .records
        .iter()
        .position(|record| record.id == id)
        .ok_or_else(|| AppError::NotFound("That backup no longer exists.".to_string()))?;
    index.records.remove(position);
    write_index(backup_dir, &index)?;
    let _ = fs::remove_file(database_path(backup_dir, id));
    let _ = fs::remove_file(manifest_path(backup_dir, id));
    Ok(())
}

pub fn restore(
    backup_dir: &Path,
    connection: &mut rusqlite::Connection,
    id: &str,
) -> AppResult<RestoreOutcome> {
    let manifest = validate_snapshot(backup_dir, id)?;
    let current_schema = schema_version(connection)?;
    if manifest.schema_version > current_schema {
        return Err(AppError::Validation(
            "That backup was created by a newer database version and cannot be restored here."
                .to_string(),
        ));
    }

    let safety = create(backup_dir, connection, Some("Safety before restore"), true)?;
    connection
        .restore(
            MAIN_DB,
            database_path(backup_dir, id),
            None::<fn(backup::Progress)>,
        )
        .map_err(|error| AppError::Internal(format!("Could not restore the backup: {error}")))?;
    db::configure(connection)?;
    let migrations_applied = db::migrations::run(connection)?;
    db::profiles::ensure_default_profile(connection)?;
    integrity_check(connection)?;

    let final_schema = schema_version(connection)?;
    let profile_count = db::profiles::list_profiles(connection)?.len();
    tracing::info!(
        backup_id = %id,
        migrations_applied,
        schema_version = final_schema,
        "Bergag database restored from a verified backup"
    );
    Ok(RestoreOutcome {
        safety_backup_id: safety.id,
        schema_version: final_schema,
        profile_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::models::ProfileDto;
    use std::collections::HashSet;

    fn test_database(path: &Path) -> rusqlite::Connection {
        db::initialize(path).expect("test database should initialize")
    }

    fn profile_count(connection: &rusqlite::Connection) -> usize {
        db::profiles::list_profiles(connection)
            .expect("profiles should be readable")
            .len()
    }

    #[test]
    fn backup_create_records_a_verifiable_snapshot_without_secrets() {
        let directory = tempfile::tempdir().expect("temp directory");
        let db_path = directory.path().join("bergag.sqlite3");
        let backup_dir = directory.path().join("backups");
        let connection = test_database(&db_path);

        let record = create(&backup_dir, &connection, Some("Nightly"), false)
            .expect("backup should be created");
        let verification = verify(&backup_dir, &record.id).expect("backup should verify");
        assert!(verification.valid);
        assert_eq!(verification.reason, None);
        assert_eq!(record.profile_count, 1);
        assert!(record.size_bytes > 0);

        let manifest = fs::read_to_string(manifest_path(&backup_dir, &record.id))
            .expect("manifest should exist");
        let lowered = manifest.to_lowercase();
        assert!(!lowered.contains("api_key"));
        assert!(!lowered.contains("password"));
        assert!(!lowered.contains("token"));
        assert_eq!(list(&backup_dir).expect("list should work").len(), 1);
    }

    #[test]
    fn restore_replaces_the_library_and_keeps_a_safety_backup() {
        let directory = tempfile::tempdir().expect("temp directory");
        let db_path = directory.path().join("bergag.sqlite3");
        let backup_dir = directory.path().join("backups");
        let mut connection = test_database(&db_path);
        let record = create(&backup_dir, &connection, None, false).expect("backup should exist");

        db::profiles::create_profile(&mut connection, "Later profile")
            .expect("extra profile should be created");
        assert_eq!(profile_count(&connection), 2);

        let outcome =
            restore(&backup_dir, &mut connection, &record.id).expect("restore should succeed");
        assert_eq!(profile_count(&connection), 1);
        assert_eq!(outcome.profile_count, 1);
        assert_eq!(outcome.schema_version, db::migrations::current_version());
        assert!(
            verify(&backup_dir, &outcome.safety_backup_id)
                .expect("safety backup should verify")
                .valid
        );
        assert_eq!(list(&backup_dir).expect("list should work").len(), 2);
    }

    #[test]
    fn corrupted_backup_is_refused_without_changing_the_current_library() {
        let directory = tempfile::tempdir().expect("temp directory");
        let db_path = directory.path().join("bergag.sqlite3");
        let backup_dir = directory.path().join("backups");
        let mut connection = test_database(&db_path);
        let record = create(&backup_dir, &connection, None, false).expect("backup should exist");

        let mut corrupted = fs::read(database_path(&backup_dir, &record.id)).expect("read backup");
        for byte in corrupted.iter_mut().take(64) {
            *byte = 0;
        }
        fs::write(database_path(&backup_dir, &record.id), corrupted).expect("write corruption");

        let verification = verify(&backup_dir, &record.id).expect("verification should return");
        assert!(!verification.valid);
        assert!(verification.reason.is_some());
        assert!(restore(&backup_dir, &mut connection, &record.id).is_err());
        assert_eq!(profile_count(&connection), 1);
        assert_eq!(list(&backup_dir).expect("list should work").len(), 1);
    }

    #[test]
    fn retention_prunes_only_automatic_backups() {
        let directory = tempfile::tempdir().expect("temp directory");
        let backup_dir = directory.path().join("backups");
        initialize(&backup_dir).expect("backup directory should initialize");
        let mut index = BackupIndex::default();
        for position in 0..8 {
            index.records.push(BackupRecord {
                id: Uuid::new_v4().to_string(),
                label: "Automatic backup".to_string(),
                created_at: format!("2026-01-01T00:00:0{position}Z"),
                schema_version: 8,
                profile_count: 1,
                size_bytes: 100,
                automatic: true,
            });
        }
        index.records.push(BackupRecord {
            id: Uuid::new_v4().to_string(),
            label: "Manual backup".to_string(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            schema_version: 8,
            profile_count: 1,
            size_bytes: 100,
            automatic: false,
        });

        prune_automatic(&backup_dir, &mut index).expect("prune should work");
        assert_eq!(
            index
                .records
                .iter()
                .filter(|record| record.automatic)
                .count(),
            AUTOMATIC_RETENTION
        );
        assert!(index.records.iter().any(|record| !record.automatic));
        let kept: HashSet<String> = index
            .records
            .iter()
            .map(|record| record.id.clone())
            .collect();
        assert!(kept.contains(&index.records.last().expect("records exist").id));
    }

    #[test]
    fn delete_removes_backup_files_and_index_entry() {
        let directory = tempfile::tempdir().expect("temp directory");
        let db_path = directory.path().join("bergag.sqlite3");
        let backup_dir = directory.path().join("backups");
        let connection = test_database(&db_path);
        let record = create(&backup_dir, &connection, None, false).expect("backup should exist");

        delete(&backup_dir, &record.id).expect("delete should work");
        assert!(list(&backup_dir).expect("list should work").is_empty());
        assert!(!database_path(&backup_dir, &record.id).exists());
        assert!(!manifest_path(&backup_dir, &record.id).exists());
        assert!(delete(&backup_dir, &record.id).is_err());
    }

    /// Run with `BERGAG_RUNTIME_DB` pointing at an installed database to verify the real
    /// create/verify/delete path against an on-disk library without changing that database.
    #[test]
    #[ignore = "requires BERGAG_RUNTIME_DB"]
    fn runtime_backup_verify_and_delete_against_installed_database() {
        let source = std::env::var_os("BERGAG_RUNTIME_DB").expect("BERGAG_RUNTIME_DB must be set");
        let directory = tempfile::tempdir().expect("temp directory");
        let copied = directory.path().join("installed-copy.sqlite3");
        fs::copy(source, &copied).expect("copy installed database");
        let connection = rusqlite::Connection::open_with_flags(
            &copied,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .expect("open installed copy");
        let backup_dir = directory.path().join("backups");

        let record = create(&backup_dir, &connection, Some("Runtime check"), false)
            .expect("create runtime backup");
        assert_eq!(record.label, "Runtime check");
        let verification = verify(&backup_dir, &record.id).expect("verify runtime backup");
        assert!(
            verification.valid,
            "{}",
            verification.reason.unwrap_or_default()
        );
        assert_eq!(list(&backup_dir).expect("list").len(), 1);
        delete(&backup_dir, &record.id).expect("delete runtime backup");
        assert!(list(&backup_dir).expect("list after delete").is_empty());
    }

    /// Run with `BERGAG_RUNTIME_DB` pointing at an installed database to verify restore,
    /// schema compatibility, and the safety backup against a copy of a real on-disk library.
    #[test]
    #[ignore = "requires BERGAG_RUNTIME_DB"]
    fn runtime_restore_against_installed_database_copy() {
        let source = PathBuf::from(std::env::var_os("BERGAG_RUNTIME_DB").expect("env var"));
        let source_hash = hash_file(&source).expect("hash installed database");
        let directory = tempfile::tempdir().expect("temp directory");
        let copied = directory.path().join("installed-copy.sqlite3");
        fs::copy(&source, &copied).expect("copy installed database");
        let mut connection = rusqlite::Connection::open(&copied).expect("open installed copy");
        let backup_dir = directory.path().join("backups");
        let original_count = profile_count(&connection);

        let record = create(&backup_dir, &connection, None, false).expect("create runtime backup");
        let marker = format!("Runtime marker {}", Uuid::new_v4());
        db::profiles::create_profile(&mut connection, &marker).expect("create marker profile");
        assert_eq!(profile_count(&connection), original_count + 1);

        let outcome =
            restore(&backup_dir, &mut connection, &record.id).expect("restore runtime backup");
        assert_eq!(outcome.profile_count, original_count);
        assert_eq!(outcome.schema_version, db::migrations::current_version());
        assert!(!db::profiles::list_profiles(&connection)
            .expect("profiles")
            .iter()
            .any(|profile| profile.name == marker));
        integrity_check(&connection).expect("restored integrity check");
        assert!(
            verify(&backup_dir, &outcome.safety_backup_id)
                .expect("safety backup verification")
                .valid
        );
        assert!(
            verify(&backup_dir, &record.id)
                .expect("source backup verification")
                .valid
        );
        assert_eq!(
            hash_file(&source).expect("rehash installed database"),
            source_hash
        );
    }

    #[test]
    fn profile_dto_shape_remains_matchable_for_manifest_validation() {
        let directory = tempfile::tempdir().expect("temp directory");
        let db_path = directory.path().join("bergag.sqlite3");
        let connection = test_database(&db_path);
        let profiles: Vec<ProfileDto> = db::profiles::list_profiles(&connection).expect("profiles");
        assert_eq!(profiles.len(), 1);
        assert!(!profiles[0].id.is_empty());
    }
}
