-- Bergag migration 0009: indexes for profile-scoped refresh and sorted library access.
-- These indexes only describe access paths; they do not seed or change library content.

CREATE INDEX IF NOT EXISTS idx_feeds_profile_status_due
  ON feeds(profile_id, status, next_refresh_at);

CREATE INDEX IF NOT EXISTS idx_feed_items_profile_sort
  ON feed_items(profile_id, COALESCE(published_at, discovered_at) DESC);

CREATE INDEX IF NOT EXISTS idx_feeds_profile_title_nocase
  ON feeds(profile_id, title COLLATE NOCASE);
