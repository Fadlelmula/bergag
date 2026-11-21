use crate::error::AppResult;
use chrono::Utc;
use rusqlite::{params, Connection};

pub struct Migration {
    pub version: i64,
    pub name: &'static str,
    pub sql: &'static str,
}

pub const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "foundation",
        sql: include_str!("../../migrations/0001_foundation.sql"),
    },
    Migration {
        version: 2,
        name: "feed_engine",
        sql: include_str!("../../migrations/0002_feed_engine.sql"),
    },
    Migration {
        version: 3,
        name: "article_system",
        sql: include_str!("../../migrations/0003_article_system.sql"),
    },
    Migration {
        version: 4,
        name: "discovery_organization",
        sql: include_str!("../../migrations/0004_discovery_organization.sql"),
    },
    Migration {
        version: 5,
        name: "search_story",
        sql: include_str!("../../migrations/0005_search_story.sql"),
    },
    Migration {
        version: 6,
        name: "dashboards_analytics",
        sql: include_str!("../../migrations/0006_dashboards_analytics.sql"),
    },
    Migration {
        version: 7,
        name: "ai_system",
        sql: include_str!("../../migrations/0007_ai_system.sql"),
    },
    Migration {
        version: 8,
        name: "integrations",
        sql: include_str!("../../migrations/0008_integrations.sql"),
    },
    Migration {
        version: 9,
        name: "performance_indexes",
        sql: include_str!("../../migrations/0009_performance_indexes.sql"),
    },
];

#[cfg(test)]
pub fn current_version() -> u32 {
    MIGRATIONS
        .last()
        .map(|migration| migration.version as u32)
        .unwrap_or(0)
}

pub fn run(connection: &mut Connection) -> AppResult<usize> {
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
           version INTEGER PRIMARY KEY,
           name TEXT NOT NULL,
           applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
         );",
    )?;

    let current: i64 = connection.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |row| row.get(0),
    )?;

    let mut applied = 0usize;
    for migration in MIGRATIONS
        .iter()
        .filter(|migration| migration.version > current)
    {
        let transaction = connection.transaction()?;
        transaction.execute_batch(migration.sql)?;
        transaction.execute(
            "INSERT INTO schema_migrations (version, name, applied_at) VALUES (?1, ?2, ?3)",
            params![migration.version, migration.name, Utc::now().to_rfc3339()],
        )?;
        transaction.commit()?;
        applied += 1;
    }

    connection.pragma_update(None, "user_version", current + applied as i64)?;
    Ok(applied)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn memory_connection() -> Connection {
        let connection = Connection::open_in_memory().unwrap();
        db::configure(&connection).unwrap();
        connection
    }

    fn apply_through(connection: &mut Connection, target: i64) {
        for migration in MIGRATIONS
            .iter()
            .filter(|migration| migration.version <= target)
        {
            let transaction = connection.transaction().unwrap();
            transaction.execute_batch(migration.sql).unwrap();
            transaction
                .execute(
                    "INSERT INTO schema_migrations (version, name, applied_at) VALUES (?1, ?2, ?3)",
                    params![migration.version, migration.name, Utc::now().to_rfc3339()],
                )
                .unwrap();
            transaction.commit().unwrap();
        }
        connection
            .pragma_update(None, "user_version", target)
            .unwrap();
    }

    fn object_exists(connection: &Connection, kind: &str, name: &str) -> bool {
        connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = ?1 AND name = ?2)",
                [kind, name],
                |row| row.get(0),
            )
            .unwrap()
    }

    #[test]
    fn applies_migrations_once() {
        let mut connection = memory_connection();
        assert_eq!(run(&mut connection).unwrap(), MIGRATIONS.len());
        assert_eq!(run(&mut connection).unwrap(), 0);

        let count: i64 = connection
            .query_row("SELECT COUNT(*) FROM schema_migrations", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, MIGRATIONS.len() as i64);
    }

    #[test]
    fn creates_expected_schema_with_foreign_keys() {
        let mut connection = memory_connection();
        run(&mut connection).unwrap();

        for table in [
            "profiles",
            "profile_settings",
            "layouts",
            "app_settings",
            "feed_folders",
            "feeds",
            "feed_items",
            "feed_refresh_logs",
            "article_sources",
            "article_states",
            "article_history",
            "article_notes",
            "article_highlights",
            "collections",
            "collection_items",
            "article_images",
            "discovery_sources",
            "labels",
            "label_assignments",
            "rules",
            "rule_runs",
            "search_presets",
            "story_groups",
            "story_group_items",
            "search_documents",
            "dashboards",
            "dashboard_widgets",
            "ai_providers",
            "ai_usage_events",
            "integration_sources",
        ] {
            let exists: bool = connection
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)",
                    [table],
                    |row| row.get(0),
                )
                .unwrap();
            assert!(exists, "missing table {table}");
        }

        let foreign_keys_enabled: i64 = connection
            .pragma_query_value(None, "foreign_keys", |row| row.get(0))
            .unwrap();
        assert_eq!(foreign_keys_enabled, 1);

        let integrity: String = connection
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .unwrap();
        assert_eq!(integrity, "ok");
    }

    #[test]
    fn upgrades_schema_seven_to_current_without_seeding_content() {
        let mut connection = memory_connection();
        apply_through(&mut connection, 7);
        assert!(!object_exists(&connection, "table", "integration_sources"));

        assert_eq!(run(&mut connection).unwrap(), 2);
        assert_eq!(run(&mut connection).unwrap(), 0);
        assert!(object_exists(&connection, "table", "integration_sources"));
        for index in [
            "idx_integration_sources_profile_kind",
            "idx_integration_sources_profile_feed",
            "idx_integration_sources_profile_status",
        ] {
            assert!(
                object_exists(&connection, "index", index),
                "missing index {index}"
            );
        }

        let migration_count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM schema_migrations WHERE version = 8 AND name = 'integrations'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(migration_count, 1);
        let user_version: i64 = connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(user_version, current_version() as i64);

        for index in [
            "idx_feeds_profile_status_due",
            "idx_feed_items_profile_sort",
            "idx_feeds_profile_title_nocase",
        ] {
            assert!(
                object_exists(&connection, "index", index),
                "missing index {index}"
            );
        }

        let source_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM integration_sources", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(source_count, 0);
        let integrity: String = connection
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .unwrap();
        assert_eq!(integrity, "ok");
    }

    #[test]
    fn integration_source_kinds_are_closed_and_secrets_are_not_stored() {
        let mut connection = memory_connection();
        run(&mut connection).unwrap();
        crate::db::profiles::ensure_default_profile(&mut connection).unwrap();
        let profile_id: String = connection
            .query_row("SELECT id FROM profiles LIMIT 1", [], |row| row.get(0))
            .unwrap();
        let now = Utc::now().to_rfc3339();
        let result = connection.execute(
            "INSERT INTO integration_sources (
               id, profile_id, kind, source_key, display_name, feed_url,
               feed_url_normalized, created_at, updated_at
             ) VALUES ('bad', ?1, 'unknown', 'key', 'Name', 'https://example.com/feed',
               'https://example.com/feed', ?2, ?2)",
            params![profile_id, now],
        );
        assert!(result.is_err());

        let columns: Vec<String> = connection
            .prepare("PRAGMA table_info(integration_sources)")
            .unwrap()
            .query_map([], |row| row.get(1))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert!(!columns.iter().any(|column| column.contains("secret")));
        assert!(!columns.iter().any(|column| column.contains("token")));
        assert!(!columns.iter().any(|column| column.contains("password")));
    }
}
