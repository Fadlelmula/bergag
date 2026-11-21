use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Validation(String),
    NotFound(String),
    Conflict(String),
    Database(String),
    Internal(String),
}

impl AppError {
    pub fn user_message(&self) -> String {
        match self {
            Self::Validation(message) => message.clone(),
            Self::NotFound(message) => message.clone(),
            Self::Conflict(message) => message.clone(),
            Self::Database(_) => {
                "A local database error occurred. Your existing data was not changed.".to_string()
            }
            Self::Internal(_) => "Bergag could not complete that operation.".to_string(),
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation(message)
            | Self::NotFound(message)
            | Self::Conflict(message)
            | Self::Database(message)
            | Self::Internal(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<rusqlite::Error> for AppError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Database(error.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        Self::Validation(format!("Invalid JSON: {error}"))
    }
}

pub type AppResult<T> = Result<T, AppError>;

/// Converts an internal error into a safe IPC error string and records details locally.
pub fn command_error(error: AppError) -> String {
    match &error {
        AppError::Database(details) => {
            tracing::error!(details = %details, "database command failed");
        }
        AppError::Internal(details) => {
            tracing::error!(details = %details, "internal command failed");
        }
        AppError::Validation(_) | AppError::NotFound(_) | AppError::Conflict(_) => {
            tracing::warn!(reason = %error, "command rejected");
        }
    }
    error.user_message()
}
