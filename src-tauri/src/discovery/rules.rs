use super::models::{RuleAction, RuleCandidate, RuleCondition, RuleDefinition, RuleRunDto};
use super::store;
use crate::articles::models::ArticleStatePatch;
use crate::articles::store as article_store;
use crate::error::{AppError, AppResult};
use chrono::Utc;
use rusqlite::Connection;

fn trust_rank(level: &str) -> u8 {
    match level {
        "low" => 1,
        "medium" => 2,
        "high" => 3,
        _ => 0,
    }
}

fn contains(haystack: &str, needle: &str) -> bool {
    haystack
        .to_lowercase()
        .contains(&needle.trim().to_lowercase())
}

fn condition_matches(condition: &RuleCondition, candidate: &RuleCandidate) -> bool {
    match condition {
        RuleCondition::TitleContains { value } => contains(&candidate.title, value),
        RuleCondition::SummaryContains { value } => contains(&candidate.summary, value),
        RuleCondition::ContentContains { value } => contains(&candidate.content_text, value),
        RuleCondition::FeedIs { feed_id } => candidate.feed_id == *feed_id,
        RuleCondition::FeedTitleContains { value } => contains(&candidate.feed_title, value),
        RuleCondition::CategoryIs { value } => {
            candidate.category.trim().eq_ignore_ascii_case(value.trim())
        }
        RuleCondition::TrustAtLeast { value } => {
            trust_rank(&candidate.trust_level) >= trust_rank(value)
        }
        RuleCondition::HasLabel { label_id } => candidate.label_ids.iter().any(|id| id == label_id),
        RuleCondition::IsRead { value } => candidate.is_read == *value,
        RuleCondition::IsSaved { value } => candidate.is_saved == *value,
        RuleCondition::IsFavorite { value } => candidate.is_favorite == *value,
        RuleCondition::IsArchived { value } => candidate.is_archived == *value,
        RuleCondition::IsQueued { value } => candidate.is_queued == *value,
    }
}

fn definition_matches(definition: &RuleDefinition, candidate: &RuleCandidate) -> bool {
    match definition.match_mode {
        super::models::MatchMode::All => definition
            .conditions
            .iter()
            .all(|condition| condition_matches(condition, candidate)),
        super::models::MatchMode::Any => definition
            .conditions
            .iter()
            .any(|condition| condition_matches(condition, candidate)),
    }
}

fn state_patch(actions: &[RuleAction]) -> ArticleStatePatch {
    let mut patch = ArticleStatePatch::default();
    for action in actions {
        match action {
            RuleAction::MarkRead { value } => patch.is_read = Some(*value),
            RuleAction::SetSaved { value } => patch.is_saved = Some(*value),
            RuleAction::SetFavorite { value } => patch.is_favorite = Some(*value),
            RuleAction::SetArchived { value } => patch.is_archived = Some(*value),
            RuleAction::SetQueued { value } => patch.is_queued = Some(*value),
            RuleAction::AddLabel { .. } | RuleAction::RemoveLabel { .. } => {}
        }
    }
    patch
}

fn state_would_change(patch: &ArticleStatePatch, candidate: &RuleCandidate) -> bool {
    patch
        .is_read
        .is_some_and(|value| value != candidate.is_read)
        || patch
            .is_saved
            .is_some_and(|value| value != candidate.is_saved)
        || patch
            .is_favorite
            .is_some_and(|value| value != candidate.is_favorite)
        || patch
            .is_archived
            .is_some_and(|value| value != candidate.is_archived)
        || patch
            .is_queued
            .is_some_and(|value| value != candidate.is_queued)
}

fn apply_label_action(
    connection: &Connection,
    profile_id: &str,
    item_id: &str,
    label_id: &str,
    add: bool,
    candidate: &RuleCandidate,
) -> AppResult<bool> {
    let exists = candidate.label_ids.iter().any(|id| id == label_id);
    if add && !exists {
        store::assign_label(connection, profile_id, label_id, item_id)?;
        return Ok(true);
    }
    if !add && exists {
        store::unassign_label(connection, profile_id, label_id, item_id)?;
        return Ok(true);
    }
    Ok(false)
}

fn execute_rule(
    connection: &mut Connection,
    profile_id: &str,
    rule_id: &str,
    definition: &RuleDefinition,
) -> AppResult<(i64, i64)> {
    let transaction = connection.transaction()?;
    let candidates = store::rule_candidates(&transaction, profile_id)?;
    let mut matched = 0i64;
    let mut changed = 0i64;

    for candidate in &candidates {
        if !definition_matches(definition, candidate) {
            continue;
        }
        matched += 1;
        let mut item_changed = false;
        let patch = state_patch(&definition.actions);
        if !patch.is_empty() {
            if state_would_change(&patch, candidate) {
                item_changed = true;
            }
            article_store::apply_patch(&transaction, profile_id, &candidate.id, &patch)?;
        }
        for action in &definition.actions {
            match action {
                RuleAction::AddLabel { label_id } => {
                    item_changed |= apply_label_action(
                        &transaction,
                        profile_id,
                        &candidate.id,
                        label_id,
                        true,
                        candidate,
                    )?;
                }
                RuleAction::RemoveLabel { label_id } => {
                    item_changed |= apply_label_action(
                        &transaction,
                        profile_id,
                        &candidate.id,
                        label_id,
                        false,
                        candidate,
                    )?;
                }
                _ => {}
            }
        }
        if item_changed {
            changed += 1;
        }
    }
    store::mark_rule_run(&transaction, profile_id, rule_id)?;
    transaction.commit()?;
    Ok((matched, changed))
}

pub fn matching_ids(
    connection: &Connection,
    profile_id: &str,
    definition: &RuleDefinition,
) -> AppResult<Vec<String>> {
    let candidates = store::rule_candidates(connection, profile_id)?;
    Ok(candidates
        .into_iter()
        .filter(|candidate| definition_matches(definition, candidate))
        .map(|candidate| candidate.id)
        .take(200)
        .collect())
}

pub fn run_rule(
    connection: &mut Connection,
    profile_id: &str,
    rule_id: &str,
) -> AppResult<RuleRunDto> {
    let rule = store::get_rule(connection, profile_id, rule_id)?;
    let definition = store::parse_and_validate_rule_json(connection, profile_id, &rule.rule_json)?;
    let started_at = Utc::now().to_rfc3339();
    let run_id = uuid::Uuid::new_v4().to_string();

    let outcome = execute_rule(connection, profile_id, rule_id, &definition);
    match outcome {
        Ok((matched, changed)) => store::record_run(
            connection,
            profile_id,
            &run_id,
            rule_id,
            &started_at,
            "success",
            matched,
            changed,
            None,
        ),
        Err(error) => {
            let message = error.to_string();
            let result = store::record_run(
                connection,
                profile_id,
                &run_id,
                rule_id,
                &started_at,
                "error",
                0,
                0,
                Some(&message),
            );
            result?;
            Err(AppError::Internal(message))
        }
    }
}

pub fn run_enabled(connection: &mut Connection, profile_id: &str) -> AppResult<Vec<RuleRunDto>> {
    let rules = store::list_rules(connection, profile_id)?;
    let mut runs = Vec::new();
    for rule in rules.into_iter().filter(|rule| rule.enabled) {
        runs.push(run_rule(connection, profile_id, &rule.id)?);
    }
    Ok(runs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate() -> RuleCandidate {
        RuleCandidate {
            id: "item".to_string(),
            title: "Rust security update".to_string(),
            summary: "A local-first release".to_string(),
            content_text: "Long article body".to_string(),
            feed_id: "feed".to_string(),
            feed_title: "Example feed".to_string(),
            category: "security".to_string(),
            trust_level: "medium".to_string(),
            is_read: false,
            is_saved: false,
            is_favorite: false,
            is_archived: false,
            is_queued: false,
            label_ids: vec!["label".to_string()],
        }
    }

    #[test]
    fn matches_all_and_any_modes() {
        let all = RuleDefinition {
            match_mode: super::super::models::MatchMode::All,
            conditions: vec![
                RuleCondition::TitleContains {
                    value: "rust".to_string(),
                },
                RuleCondition::TrustAtLeast {
                    value: "medium".to_string(),
                },
            ],
            actions: vec![RuleAction::MarkRead { value: true }],
        };
        let any = RuleDefinition {
            match_mode: super::super::models::MatchMode::Any,
            conditions: vec![
                RuleCondition::CategoryIs {
                    value: "other".to_string(),
                },
                RuleCondition::HasLabel {
                    label_id: "label".to_string(),
                },
            ],
            actions: vec![RuleAction::MarkRead { value: true }],
        };
        assert!(definition_matches(&all, &candidate()));
        assert!(definition_matches(&any, &candidate()));
    }

    #[test]
    fn builds_state_patch_from_actions() {
        let patch = state_patch(&[
            RuleAction::MarkRead { value: true },
            RuleAction::SetQueued { value: true },
            RuleAction::AddLabel {
                label_id: "label".to_string(),
            },
        ]);
        assert_eq!(patch.is_read, Some(true));
        assert_eq!(patch.is_queued, Some(true));
        assert_eq!(patch.is_saved, None);
        assert!(state_would_change(&patch, &candidate()));
    }

    #[test]
    fn runs_rules_against_local_articles_and_records_history() {
        let mut connection = Connection::open_in_memory().unwrap();
        crate::db::configure(&connection).unwrap();
        crate::db::migrations::run(&mut connection).unwrap();
        crate::db::profiles::ensure_default_profile(&mut connection).unwrap();
        let first = crate::db::settings::get_active_profile_id(&connection).unwrap();
        let second = crate::db::profiles::create_profile(&mut connection, "Other")
            .unwrap()
            .id;

        let feed = crate::db::feeds::NewFeed {
            title: "Security feed",
            site_url: None,
            feed_url: "https://example.com/feed.xml",
            feed_url_normalized: "https://example.com/feed.xml",
            feed_type: "rss",
            description: None,
            language: None,
            etag: None,
            last_modified: None,
            folder_id: None,
        };
        crate::db::feeds::subscribe(&mut connection, &first, "feed-1", &feed, &[]).unwrap();
        crate::db::feeds::set_organization(&connection, &first, "feed-1", Some("Security"), "high")
            .unwrap();
        let item = crate::db::feeds::NewItem {
            guid: "item-1",
            title: "Rust security update",
            url: None,
            author: None,
            summary: Some("A local-first release"),
            content_html: None,
            published_at: None,
        };
        crate::db::feeds::insert_items(&connection, &first, "feed-1", &[item]).unwrap();
        let item_id: String = connection
            .query_row(
                "SELECT id FROM feed_items WHERE profile_id = ?1",
                [&first],
                |row| row.get(0),
            )
            .unwrap();

        let label =
            store::create_label(&connection, &first, "label-1", "topic", "Rust", None, None)
                .unwrap();
        let rule_json = format!(
            r#"{{"matchMode":"all","conditions":[{{"field":"titleContains","value":"rust"}},{{"field":"trustAtLeast","value":"medium"}}],"actions":[{{"action":"markRead","value":true}},{{"action":"addLabel","labelId":"{}"}}]}}"#,
            label.id
        );
        store::create_rule(
            &connection,
            &first,
            "rule-1",
            "Security triage",
            None,
            &rule_json,
            true,
            true,
        )
        .unwrap();

        let definition =
            store::parse_and_validate_rule_json(&connection, &first, &rule_json).unwrap();
        assert_eq!(
            matching_ids(&connection, &first, &definition).unwrap(),
            vec![item_id.clone()]
        );

        let run = run_rule(&mut connection, &first, "rule-1").unwrap();
        assert_eq!(run.status, "success");
        assert_eq!(run.matched_count, 1);
        assert_eq!(run.changed_count, 1);

        let is_read: bool = connection
            .query_row(
                "SELECT is_read FROM article_states WHERE profile_id = ?1 AND item_id = ?2",
                rusqlite::params![first, item_id],
                |row| row.get(0),
            )
            .unwrap();
        assert!(is_read);
        assert_eq!(
            store::list_item_labels(&connection, &first, &item_id)
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            store::list_rule_runs(&connection, &first, "rule-1", 10)
                .unwrap()
                .len(),
            1
        );
        assert!(store::list_rules(&connection, &second).unwrap().is_empty());
        assert!(run_rule(&mut connection, &second, "rule-1").is_err());
    }
}
