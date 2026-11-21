use super::models::{DashboardDto, DashboardWidgetDto};
use crate::error::{AppError, AppResult};
use crate::validation::{normalize_display_name, normalize_optional_text};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};

const MAX_DASHBOARDS: i64 = 12;
const MAX_WIDGETS: i64 = 24;

pub fn widget_defaults(kind: &str) -> AppResult<(&'static str, i64, i64)> {
    match kind {
        "unread_by_feed" => Ok(("Unread by feed", 7, 5)),
        "recent_articles" => Ok(("Recently discovered", 7, 5)),
        "reading_activity" => Ok(("Reading activity", 30, 5)),
        "top_labels" => Ok(("Top labels", 30, 5)),
        "feed_health" => Ok(("Feed health", 30, 5)),
        "storage" => Ok(("Local storage", 30, 5)),
        "briefing" => Ok(("Briefing", 7, 5)),
        "reading_queue" => Ok(("Reading queue", 30, 5)),
        _ => Err(AppError::Validation("Unknown widget type.".to_string())),
    }
}

fn map_dashboard(row: &rusqlite::Row<'_>) -> rusqlite::Result<DashboardDto> {
    Ok(DashboardDto {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        is_default: row.get::<_, i64>(3)? != 0,
        widget_count: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

fn map_widget(row: &rusqlite::Row<'_>) -> rusqlite::Result<DashboardWidgetDto> {
    Ok(DashboardWidgetDto {
        id: row.get(0)?,
        dashboard_id: row.get(1)?,
        kind: row.get(2)?,
        title: row.get(3)?,
        days: row.get(4)?,
        item_limit: row.get(5)?,
        sort_order: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

fn find_dashboard(
    connection: &Connection,
    profile_id: &str,
    id: &str,
) -> AppResult<Option<DashboardDto>> {
    connection
        .query_row(
            "SELECT d.id, d.name, d.description, d.is_default,
               (SELECT COUNT(*) FROM dashboard_widgets w
                 WHERE w.profile_id = d.profile_id AND w.dashboard_id = d.id),
               d.created_at, d.updated_at
             FROM dashboards d WHERE d.profile_id = ?1 AND d.id = ?2",
            params![profile_id, id],
            map_dashboard,
        )
        .optional()
        .map_err(AppError::from)
}

fn require_dashboard(
    connection: &Connection,
    profile_id: &str,
    id: &str,
) -> AppResult<DashboardDto> {
    find_dashboard(connection, profile_id, id)?
        .ok_or_else(|| AppError::NotFound("That dashboard no longer exists.".to_string()))
}

pub fn list_dashboards(connection: &Connection, profile_id: &str) -> AppResult<Vec<DashboardDto>> {
    let mut statement = connection.prepare(
        "SELECT d.id, d.name, d.description, d.is_default,
           (SELECT COUNT(*) FROM dashboard_widgets w
             WHERE w.profile_id = d.profile_id AND w.dashboard_id = d.id),
           d.created_at, d.updated_at
         FROM dashboards d WHERE d.profile_id = ?1
         ORDER BY d.is_default DESC, d.name COLLATE NOCASE",
    )?;
    let rows = statement.query_map([profile_id], map_dashboard)?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(AppError::from)
}

pub fn save_dashboard(
    connection: &mut Connection,
    profile_id: &str,
    id: Option<&str>,
    name: &str,
    description: Option<&str>,
) -> AppResult<DashboardDto> {
    let (name, name_normalized) = normalize_display_name(name, "Dashboard name")?;
    let description = normalize_optional_text(description, "Description", 240)?;
    let transaction = connection.transaction()?;
    let timestamp = Utc::now().to_rfc3339();

    let dashboard = if let Some(id) = id {
        require_dashboard(&transaction, profile_id, id)?;
        transaction.execute(
            "UPDATE dashboards SET name = ?1, name_normalized = ?2,
               description = ?3, updated_at = ?4
             WHERE profile_id = ?5 AND id = ?6",
            params![
                name,
                name_normalized,
                description,
                timestamp,
                profile_id,
                id
            ],
        )?;
        require_dashboard(&transaction, profile_id, id)?
    } else {
        let count: i64 = transaction.query_row(
            "SELECT COUNT(*) FROM dashboards WHERE profile_id = ?1",
            [profile_id],
            |row| row.get(0),
        )?;
        if count >= MAX_DASHBOARDS {
            return Err(AppError::Validation(format!(
                "You can keep at most {MAX_DASHBOARDS} dashboards."
            )));
        }
        let id = uuid::Uuid::new_v4().to_string();
        let is_default = count == 0;
        transaction.execute(
            "INSERT INTO dashboards (
               id, profile_id, name, name_normalized, description, is_default,
               created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
            params![
                id,
                profile_id,
                name,
                name_normalized,
                description,
                i64::from(is_default),
                timestamp
            ],
        )?;
        require_dashboard(&transaction, profile_id, &id)?
    };
    transaction.commit()?;
    Ok(dashboard)
}

pub fn delete_dashboard(connection: &mut Connection, profile_id: &str, id: &str) -> AppResult<()> {
    let transaction = connection.transaction()?;
    let dashboard = require_dashboard(&transaction, profile_id, id)?;
    transaction.execute(
        "DELETE FROM dashboards WHERE profile_id = ?1 AND id = ?2",
        params![profile_id, id],
    )?;
    if dashboard.is_default {
        let next: Option<String> = transaction
            .query_row(
                "SELECT id FROM dashboards WHERE profile_id = ?1 ORDER BY created_at, name LIMIT 1",
                [profile_id],
                |row| row.get(0),
            )
            .optional()?;
        if let Some(next) = next {
            transaction.execute(
                "UPDATE dashboards SET is_default = 1 WHERE profile_id = ?1 AND id = ?2",
                params![profile_id, next],
            )?;
        }
    }
    transaction.commit()?;
    Ok(())
}

pub fn set_default_dashboard(
    connection: &mut Connection,
    profile_id: &str,
    id: &str,
) -> AppResult<DashboardDto> {
    let transaction = connection.transaction()?;
    require_dashboard(&transaction, profile_id, id)?;
    transaction.execute(
        "UPDATE dashboards SET is_default = 0 WHERE profile_id = ?1 AND is_default = 1",
        [profile_id],
    )?;
    let timestamp = Utc::now().to_rfc3339();
    transaction.execute(
        "UPDATE dashboards SET is_default = 1, updated_at = ?1
         WHERE profile_id = ?2 AND id = ?3",
        params![timestamp, profile_id, id],
    )?;
    let dashboard = require_dashboard(&transaction, profile_id, id)?;
    transaction.commit()?;
    Ok(dashboard)
}

pub fn list_widgets(
    connection: &Connection,
    profile_id: &str,
    dashboard_id: &str,
) -> AppResult<Vec<DashboardWidgetDto>> {
    require_dashboard(connection, profile_id, dashboard_id)?;
    let mut statement = connection.prepare(
        "SELECT id, dashboard_id, kind, title, days, item_limit, sort_order,
           created_at, updated_at
         FROM dashboard_widgets
         WHERE profile_id = ?1 AND dashboard_id = ?2
         ORDER BY sort_order, created_at",
    )?;
    let rows = statement.query_map(params![profile_id, dashboard_id], map_widget)?;
    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(AppError::from)
}

pub fn add_widget(
    connection: &mut Connection,
    profile_id: &str,
    dashboard_id: &str,
    kind: &str,
) -> AppResult<DashboardWidgetDto> {
    let (title, days, item_limit) = widget_defaults(kind)?;
    let transaction = connection.transaction()?;
    require_dashboard(&transaction, profile_id, dashboard_id)?;
    let (count, max_order): (i64, i64) = transaction.query_row(
        "SELECT COUNT(*), COALESCE(MAX(sort_order), -1)
         FROM dashboard_widgets WHERE profile_id = ?1 AND dashboard_id = ?2",
        params![profile_id, dashboard_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    if count >= MAX_WIDGETS {
        return Err(AppError::Validation(format!(
            "A dashboard can hold at most {MAX_WIDGETS} widgets."
        )));
    }
    let id = uuid::Uuid::new_v4().to_string();
    let timestamp = Utc::now().to_rfc3339();
    transaction.execute(
        "INSERT INTO dashboard_widgets (
           id, dashboard_id, profile_id, kind, title, days, item_limit,
           sort_order, created_at, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
        params![
            id,
            dashboard_id,
            profile_id,
            kind,
            title,
            days,
            item_limit,
            (max_order + 1).min(1000),
            timestamp
        ],
    )?;
    let widget = transaction
        .query_row(
            "SELECT id, dashboard_id, kind, title, days, item_limit, sort_order,
               created_at, updated_at
             FROM dashboard_widgets WHERE profile_id = ?1 AND id = ?2",
            params![profile_id, id],
            map_widget,
        )
        .optional()?
        .ok_or_else(|| AppError::Internal("Widget was not saved.".to_string()))?;
    transaction.commit()?;
    Ok(widget)
}

fn validate_widget_update(
    title: &str,
    days: i64,
    item_limit: i64,
    sort_order: i64,
) -> AppResult<(String, i64, i64, i64)> {
    let title = title.trim();
    if title.is_empty() || title.chars().count() > 96 || title.chars().any(char::is_control) {
        return Err(AppError::Validation(
            "Widget title must be 1 to 96 characters without control characters.".to_string(),
        ));
    }
    if !(1..=365).contains(&days) {
        return Err(AppError::Validation(
            "Widget range must be between 1 and 365 days.".to_string(),
        ));
    }
    if !(1..=20).contains(&item_limit) {
        return Err(AppError::Validation(
            "Widget item limit must be between 1 and 20.".to_string(),
        ));
    }
    if !(0..=1000).contains(&sort_order) {
        return Err(AppError::Validation(
            "Widget position must be between 0 and 1000.".to_string(),
        ));
    }
    Ok((title.to_string(), days, item_limit, sort_order))
}

pub fn update_widget(
    connection: &Connection,
    profile_id: &str,
    id: &str,
    title: &str,
    days: i64,
    item_limit: i64,
    sort_order: i64,
) -> AppResult<DashboardWidgetDto> {
    let (title, days, item_limit, sort_order) =
        validate_widget_update(title, days, item_limit, sort_order)?;
    let belongs: bool = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM dashboard_widgets WHERE profile_id = ?1 AND id = ?2)",
        params![profile_id, id],
        |row| row.get(0),
    )?;
    if !belongs {
        return Err(AppError::NotFound(
            "That widget no longer exists.".to_string(),
        ));
    }
    let timestamp = Utc::now().to_rfc3339();
    connection.execute(
        "UPDATE dashboard_widgets SET title = ?1, days = ?2, item_limit = ?3,
           sort_order = ?4, updated_at = ?5
         WHERE profile_id = ?6 AND id = ?7",
        params![title, days, item_limit, sort_order, timestamp, profile_id, id],
    )?;
    connection
        .query_row(
            "SELECT id, dashboard_id, kind, title, days, item_limit, sort_order,
               created_at, updated_at
             FROM dashboard_widgets WHERE profile_id = ?1 AND id = ?2",
            params![profile_id, id],
            map_widget,
        )
        .map_err(AppError::from)
}

pub fn delete_widget(connection: &Connection, profile_id: &str, id: &str) -> AppResult<()> {
    let deleted = connection.execute(
        "DELETE FROM dashboard_widgets WHERE profile_id = ?1 AND id = ?2",
        params![profile_id, id],
    )?;
    if deleted == 0 {
        return Err(AppError::NotFound(
            "That widget no longer exists.".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{self, profiles};

    fn setup() -> (Connection, String, String) {
        let mut connection = Connection::open_in_memory().unwrap();
        db::configure(&connection).unwrap();
        db::migrations::run(&mut connection).unwrap();
        profiles::ensure_default_profile(&mut connection).unwrap();
        let first = db::settings::get_active_profile_id(&connection).unwrap();
        let second = profiles::create_profile(&mut connection, "Other")
            .unwrap()
            .id;
        (connection, first, second)
    }

    #[test]
    fn dashboards_are_profile_scoped_and_validated() {
        let (mut connection, first, second) = setup();
        let dashboard =
            save_dashboard(&mut connection, &first, None, "  Daily   Brief ", None).unwrap();
        assert_eq!(dashboard.name, "Daily Brief");
        assert!(dashboard.is_default);
        assert!(save_dashboard(&mut connection, &first, None, "daily brief", None).is_err());
        assert!(save_dashboard(&mut connection, &first, None, "", None).is_err());
        assert!(list_dashboards(&connection, &second).unwrap().is_empty());

        let renamed = save_dashboard(
            &mut connection,
            &first,
            Some(&dashboard.id),
            "Weekly Review",
            Some("Local notes"),
        )
        .unwrap();
        assert_eq!(renamed.name, "Weekly Review");
        assert_eq!(renamed.description.as_deref(), Some("Local notes"));

        let other = save_dashboard(&mut connection, &second, None, "Separate", None).unwrap();
        assert!(set_default_dashboard(&mut connection, &second, &other.id).is_ok());
        assert!(set_default_dashboard(&mut connection, &second, &dashboard.id).is_err());
        let widget = add_widget(&mut connection, &first, &dashboard.id, "storage").unwrap();
        assert_eq!(
            list_widgets(&connection, &first, &dashboard.id)
                .unwrap()
                .len(),
            1
        );

        delete_dashboard(&mut connection, &first, &dashboard.id).unwrap();
        assert!(list_dashboards(&connection, &first).unwrap().is_empty());
        assert!(delete_widget(&connection, &first, &widget.id).is_err());
        assert_eq!(list_dashboards(&connection, &second).unwrap().len(), 1);
    }

    #[test]
    fn widget_defaults_updates_and_orders_are_validated() {
        let (mut connection, profile_id, _) = setup();
        let dashboard =
            save_dashboard(&mut connection, &profile_id, None, "Overview", None).unwrap();
        assert!(widget_defaults("not_a_widget").is_err());
        let first = add_widget(
            &mut connection,
            &profile_id,
            &dashboard.id,
            "reading_activity",
        )
        .unwrap();
        let second =
            add_widget(&mut connection, &profile_id, &dashboard.id, "feed_health").unwrap();
        assert_eq!(first.sort_order, 0);
        assert_eq!(second.sort_order, 1);
        assert_eq!(first.days, 30);

        assert!(update_widget(&mut connection, &profile_id, &first.id, "", 30, 5, 0).is_err());
        assert!(update_widget(&mut connection, &profile_id, &first.id, "Good", 0, 5, 0).is_err());
        let updated =
            update_widget(&mut connection, &profile_id, &first.id, "Good", 7, 3, 2).unwrap();
        assert_eq!(updated.title, "Good");
        assert_eq!(updated.days, 7);
        assert_eq!(updated.sort_order, 2);
        assert!(update_widget(&mut connection, &profile_id, &first.id, "Good", 7, 3, 0).is_ok());

        delete_widget(&connection, &profile_id, &second.id).unwrap();
        let widgets = list_widgets(&connection, &profile_id, &dashboard.id).unwrap();
        assert_eq!(widgets.len(), 1);
        assert_eq!(widgets[0].id, first.id);
    }
}
