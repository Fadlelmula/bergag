use crate::db::settings;
use crate::error::AppResult;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub const CLOSE_BEHAVIOR_KEY: &str = "desktop.close_behavior";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CloseBehavior {
    Quit,
    Tray,
}

impl CloseBehavior {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Quit => "quit",
            Self::Tray => "tray",
        }
    }
}

pub fn close_behavior(connection: &Connection) -> AppResult<CloseBehavior> {
    let value = settings::get_global_setting(connection, CLOSE_BEHAVIOR_KEY)?;
    match value.as_ref().and_then(|json| json.as_str()) {
        Some("tray") => Ok(CloseBehavior::Tray),
        _ => Ok(CloseBehavior::Quit),
    }
}

pub fn set_close_behavior(connection: &Connection, behavior: CloseBehavior) -> AppResult<()> {
    settings::set_global_setting(connection, CLOSE_BEHAVIOR_KEY, &json!(behavior.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    #[test]
    fn close_behavior_defaults_to_quit_and_persists_selected_value() {
        let directory = tempfile::tempdir().expect("temp directory");
        let connection =
            db::initialize(&directory.path().join("bergag.sqlite3")).expect("database");
        assert_eq!(
            close_behavior(&connection).expect("default"),
            CloseBehavior::Quit
        );

        set_close_behavior(&connection, CloseBehavior::Tray).expect("save tray");
        assert_eq!(
            close_behavior(&connection).expect("saved"),
            CloseBehavior::Tray
        );

        set_close_behavior(&connection, CloseBehavior::Quit).expect("save quit");
        assert_eq!(
            close_behavior(&connection).expect("saved"),
            CloseBehavior::Quit
        );
    }

    #[test]
    fn invalid_close_behavior_is_ignored() {
        let directory = tempfile::tempdir().expect("temp directory");
        let connection =
            db::initialize(&directory.path().join("bergag.sqlite3")).expect("database");
        settings::set_global_setting(&connection, CLOSE_BEHAVIOR_KEY, &json!("unknown"))
            .expect("invalid setting");
        assert_eq!(
            close_behavior(&connection).expect("fallback"),
            CloseBehavior::Quit
        );
    }

    #[test]
    fn close_behavior_as_str_is_stable() {
        assert_eq!(CloseBehavior::Quit.as_str(), "quit");
        assert_eq!(CloseBehavior::Tray.as_str(), "tray");
    }
}
