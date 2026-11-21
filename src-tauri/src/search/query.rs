use crate::error::{AppError, AppResult};

pub const SCOPES: &[&str] = &["all", "unread", "saved", "favorites", "archive", "queue"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Filter {
    Feed(String),
    Folder(String),
    Label(String),
    Author(String),
    Category(String),
    Trust(String),
    Before(String),
    After(String),
    State { column: &'static str, value: bool },
    HasNotes(bool),
    HasHighlights(bool),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchPlan {
    pub fts: Option<String>,
    pub scope: String,
    pub filters: Vec<Filter>,
}

fn quote_fts(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn strip_quotes(value: &str) -> AppResult<&str> {
    let trimmed = value.trim();
    if trimmed.starts_with('"') || trimmed.ends_with('"') {
        if trimmed.len() < 2 || !trimmed.starts_with('"') || !trimmed.ends_with('"') {
            return Err(AppError::Validation(
                "Close the quoted phrase before searching.".to_string(),
            ));
        }
        return Ok(&trimmed[1..trimmed.len() - 1]);
    }
    Ok(trimmed)
}

fn split_terms(input: &str) -> AppResult<Vec<String>> {
    let mut terms = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    for character in input.chars() {
        match character {
            '"' => {
                quoted = !quoted;
                current.push(character);
            }
            value if value.is_whitespace() && !quoted => {
                if !current.is_empty() {
                    terms.push(std::mem::take(&mut current));
                }
            }
            value => current.push(value),
        }
    }
    if quoted {
        return Err(AppError::Validation(
            "Close the quoted phrase before searching.".to_string(),
        ));
    }
    if !current.is_empty() {
        terms.push(current);
    }
    Ok(terms)
}

fn split_operator(term: &str) -> Option<(&str, &str)> {
    let mut quoted = false;
    for (index, character) in term.char_indices() {
        match character {
            '"' => quoted = !quoted,
            ':' if !quoted => {
                if index == 0 {
                    return None;
                }
                return Some((&term[..index], &term[index + 1..]));
            }
            _ => {}
        }
    }
    None
}

fn state_column(value: &str) -> Option<(&'static str, bool)> {
    match value {
        "read" => Some(("is_read", true)),
        "unread" => Some(("is_read", false)),
        "saved" => Some(("is_saved", true)),
        "unsaved" => Some(("is_saved", false)),
        "favorite" | "favorited" => Some(("is_favorite", true)),
        "archived" => Some(("is_archived", true)),
        "unarchived" => Some(("is_archived", false)),
        "queued" => Some(("is_queued", true)),
        "unqueued" => Some(("is_queued", false)),
        _ => None,
    }
}

fn validate_date(value: &str, operator: &str) -> AppResult<String> {
    chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| {
        AppError::Validation(format!("{operator}: expects a date like 2026-11-20."))
    })?;
    Ok(value.to_string())
}

pub fn parse(raw: &str, scope: &str) -> AppResult<SearchPlan> {
    let query = raw.trim();
    if query.is_empty() {
        return Err(AppError::Validation("Enter a search query.".to_string()));
    }
    if query.chars().count() > 500 {
        return Err(AppError::Validation(
            "Search queries must be 500 characters or fewer.".to_string(),
        ));
    }
    if !SCOPES.contains(&scope) {
        return Err(AppError::Validation(
            "Search scope must be all, unread, saved, favorites, archive, or queue.".to_string(),
        ));
    }

    let mut positives: Vec<String> = Vec::new();
    let mut negatives: Vec<String> = Vec::new();
    let mut filters: Vec<Filter> = Vec::new();

    for raw_term in split_terms(query)? {
        let (negative, term) = match raw_term.strip_prefix('-') {
            Some(rest) if !rest.is_empty() => (true, rest),
            _ => (false, raw_term.as_str()),
        };
        let Some((field, raw_value)) = split_operator(term) else {
            let value = strip_quotes(term)?;
            if value.trim().is_empty() {
                continue;
            }
            if value.chars().count() > 200 {
                return Err(AppError::Validation(
                    "Search terms must be 200 characters or fewer.".to_string(),
                ));
            }
            if negative {
                negatives.push(quote_fts(value));
            } else {
                positives.push(quote_fts(value));
            }
            continue;
        };

        let field = field.to_ascii_lowercase();
        let value = strip_quotes(raw_value)?;
        if value.trim().is_empty() {
            return Err(AppError::Validation(format!("{field}: needs a value.")));
        }
        if value.chars().count() > 200 {
            return Err(AppError::Validation(format!(
                "{field}: values must be 200 characters or fewer."
            )));
        }

        match field.as_str() {
            "title" => {
                let expression = format!("title : {}", quote_fts(value));
                if negative {
                    negatives.push(expression);
                } else {
                    positives.push(expression);
                }
            }
            "feed" => filters.push(Filter::Feed(value.to_string())),
            "folder" => filters.push(Filter::Folder(value.to_string())),
            "tag" | "topic" | "label" => filters.push(Filter::Label(value.to_string())),
            "author" => filters.push(Filter::Author(value.to_string())),
            "category" => filters.push(Filter::Category(value.to_string())),
            "trust" => {
                let normalized = value.to_ascii_lowercase();
                if !["untrusted", "low", "medium", "high"].contains(&normalized.as_str()) {
                    return Err(AppError::Validation(
                        "trust: expects untrusted, low, medium, or high.".to_string(),
                    ));
                }
                filters.push(Filter::Trust(normalized));
            }
            "before" => filters.push(Filter::Before(validate_date(value, "before")?)),
            "after" => filters.push(Filter::After(validate_date(value, "after")?)),
            "is" => {
                let normalized = value.to_ascii_lowercase();
                let Some((column, mut state)) = state_column(&normalized) else {
                    return Err(AppError::Validation(
                        "is: expects read, unread, saved, unsaved, favorite, archived, or queued."
                            .to_string(),
                    ));
                };
                if negative {
                    state = !state;
                }
                filters.push(Filter::State {
                    column,
                    value: state,
                });
            }
            "has" => {
                let normalized = value.to_ascii_lowercase();
                match normalized.as_str() {
                    "notes" => filters.push(Filter::HasNotes(!negative)),
                    "highlights" => filters.push(Filter::HasHighlights(!negative)),
                    _ => {
                        return Err(AppError::Validation(
                            "has: expects notes or highlights.".to_string(),
                        ))
                    }
                }
            }
            _ => {
                // An unknown prefix is treated as ordinary text, so URLs and
                // colon-containing words still work.
                let value = term;
                if negative {
                    negatives.push(quote_fts(value));
                } else {
                    positives.push(quote_fts(value));
                }
            }
        }
    }

    if positives.is_empty() && negatives.is_empty() && filters.is_empty() {
        return Err(AppError::Validation(
            "Enter a search term or a supported filter.".to_string(),
        ));
    }
    if positives.is_empty() && !negatives.is_empty() {
        return Err(AppError::Validation(
            "Add a positive search term; negated terms need something to match.".to_string(),
        ));
    }

    let mut fts = positives.join(" AND ");
    for negative in negatives {
        fts.push_str(" NOT ");
        fts.push_str(&negative);
    }

    Ok(SearchPlan {
        fts: if fts.is_empty() { None } else { Some(fts) },
        scope: scope.to_string(),
        filters,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_phrases_operators_and_negation() {
        let plan = parse(
            r#""local first" rust -spam feed:"The Verge" is:unread has:notes after:2026-01-01"#,
            "all",
        )
        .unwrap();
        assert_eq!(plan.scope, "all");
        assert_eq!(
            plan.fts.as_deref(),
            Some("\"local first\" AND \"rust\" NOT \"spam\"")
        );
        assert!(plan
            .filters
            .contains(&Filter::Feed("The Verge".to_string())));
        assert!(plan.filters.contains(&Filter::State {
            column: "is_read",
            value: false
        }));
        assert!(plan.filters.contains(&Filter::HasNotes(true)));
        assert!(plan
            .filters
            .contains(&Filter::After("2026-01-01".to_string())));
    }

    #[test]
    fn supports_filter_only_searches() {
        let plan = parse("is:saved tag:Research", "all").unwrap();
        assert!(plan.fts.is_none());
        assert!(plan.filters.contains(&Filter::State {
            column: "is_saved",
            value: true
        }));
        assert!(plan
            .filters
            .contains(&Filter::Label("Research".to_string())));
    }

    #[test]
    fn rejects_invalid_operators_and_quotes() {
        assert!(parse("", "all").is_err());
        assert!(parse("rust", "unknown").is_err());
        assert!(parse("is:sideways", "all").is_err());
        assert!(parse("has:images", "all").is_err());
        assert!(parse("trust:maybe", "all").is_err());
        assert!(parse("after:yesterday", "all").is_err());
        assert!(parse(r#""unclosed"#, "all").is_err());
        assert!(parse("-spam", "all").is_err());
        assert!(parse("feed:", "all").is_err());
    }
}
