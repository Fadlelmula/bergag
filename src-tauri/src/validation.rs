use crate::error::{AppError, AppResult};
use serde_json::Value;

pub fn normalize_display_name(raw: &str, label: &str) -> AppResult<(String, String)> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation(format!("{label} cannot be empty.")));
    }
    if trimmed.chars().count() > 64 {
        return Err(AppError::Validation(format!(
            "{label} must be 64 characters or fewer."
        )));
    }
    if trimmed.chars().any(|character| character.is_control()) {
        return Err(AppError::Validation(format!(
            "{label} cannot contain control characters."
        )));
    }

    let collapsed = trimmed.split_whitespace().collect::<Vec<_>>().join(" ");
    let normalized = collapsed.to_lowercase();
    Ok((collapsed, normalized))
}

pub fn normalize_trust_level(raw: &str) -> AppResult<String> {
    match raw {
        "untrusted" | "low" | "medium" | "high" => Ok(raw.to_string()),
        _ => Err(AppError::Validation(
            "Trust level must be untrusted, low, medium, or high.".to_string(),
        )),
    }
}

pub fn normalize_optional_text(
    raw: Option<&str>,
    label: &str,
    maximum: usize,
) -> AppResult<Option<String>> {
    let Some(value) = raw else {
        return Ok(None);
    };
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if trimmed.chars().count() > maximum {
        return Err(AppError::Validation(format!(
            "{label} must be {maximum} characters or fewer."
        )));
    }
    if trimmed.chars().any(|character| character.is_control()) {
        return Err(AppError::Validation(format!(
            "{label} cannot contain control characters."
        )));
    }
    Ok(Some(trimmed.to_string()))
}

pub fn validate_setting_key(key: &str) -> AppResult<()> {
    let bytes = key.as_bytes();
    if bytes.is_empty() || bytes.len() > 64 {
        return Err(AppError::Validation(
            "Setting keys must be between 1 and 64 characters.".to_string(),
        ));
    }
    let first = bytes[0];
    if !first.is_ascii_lowercase() {
        return Err(AppError::Validation(
            "Setting keys must start with a lowercase letter.".to_string(),
        ));
    }
    if bytes.iter().any(|byte| {
        !(byte.is_ascii_lowercase()
            || byte.is_ascii_digit()
            || *byte == b'.'
            || *byte == b'_'
            || *byte == b'-')
    }) {
        return Err(AppError::Validation(
            "Setting keys may contain lowercase letters, digits, dots, underscores, and hyphens."
                .to_string(),
        ));
    }
    Ok(())
}

pub fn serialize_json_value(value: &Value, max_bytes: usize, label: &str) -> AppResult<String> {
    let serialized = serde_json::to_string(value)
        .map_err(|error| AppError::Validation(format!("Invalid {label}: {error}")))?;
    if serialized.len() > max_bytes {
        return Err(AppError::Validation(format!(
            "{label} is too large (maximum {max_bytes} bytes)."
        )));
    }
    Ok(serialized)
}

pub fn validate_layout_json(layout_json: &str) -> AppResult<Value> {
    if layout_json.len() > 512_000 {
        return Err(AppError::Validation(
            "Layout is too large (maximum 512 KB).".to_string(),
        ));
    }
    let value: Value = serde_json::from_str(layout_json)
        .map_err(|error| AppError::Validation(format!("Invalid layout JSON: {error}")))?;
    let object = value
        .as_object()
        .ok_or_else(|| AppError::Validation("Layout must be a JSON object.".to_string()))?;
    if !object.contains_key("version") {
        return Err(AppError::Validation(
            "Layout is missing a version.".to_string(),
        ));
    }
    if !object.contains_key("panels") {
        return Err(AppError::Validation(
            "Layout is missing panels configuration.".to_string(),
        ));
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn validates_setting_keys() {
        assert!(validate_setting_key("appearance.theme").is_ok());
        assert!(validate_setting_key("appearance-theme_1").is_ok());
        assert!(validate_setting_key("Appearance.theme").is_err());
        assert!(validate_setting_key("has space").is_err());
        assert!(validate_setting_key("").is_err());
    }

    #[test]
    fn normalizes_names_without_control_characters() {
        let (display, normalized) = normalize_display_name("  Research   Notes ", "Name").unwrap();
        assert_eq!(display, "Research Notes");
        assert_eq!(normalized, "research notes");
        assert!(normalize_display_name("Bad\tName", "Name").is_err());
    }

    #[test]
    fn validates_layout_shape_and_size() {
        assert!(validate_layout_json(r#"{"version":1,"panels":{}}"#).is_ok());
        assert!(validate_layout_json(r#"{"version":1}"#).is_err());
        assert!(validate_layout_json(r#"[]"#).is_err());
        let huge = json!({"version": 1, "panels": "x".repeat(513_000)}).to_string();
        assert!(validate_layout_json(&huge).is_err());
    }
}
