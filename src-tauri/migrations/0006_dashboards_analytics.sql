-- Bergag migration 0006: user-authored dashboards and widgets.
-- Reading, library, feed health, and storage analytics are computed on demand from existing local tables.
-- All statements are executed inside a transaction by the migration runner.

CREATE TABLE IF NOT EXISTS dashboards (
  id TEXT PRIMARY KEY,
  profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
  name TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 64),
  name_normalized TEXT NOT NULL CHECK (length(name_normalized) BETWEEN 1 AND 64),
  description TEXT CHECK (description IS NULL OR length(description) BETWEEN 1 AND 240),
  is_default INTEGER NOT NULL DEFAULT 0 CHECK (is_default IN (0, 1)),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE (profile_id, id),
  UNIQUE (profile_id, name_normalized)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_dashboards_default
  ON dashboards(profile_id) WHERE is_default = 1;

CREATE TABLE IF NOT EXISTS dashboard_widgets (
  id TEXT PRIMARY KEY,
  dashboard_id TEXT NOT NULL,
  profile_id TEXT NOT NULL,
  kind TEXT NOT NULL CHECK (kind IN (
    'unread_by_feed',
    'recent_articles',
    'reading_activity',
    'top_labels',
    'feed_health',
    'storage',
    'briefing',
    'reading_queue'
  )),
  title TEXT NOT NULL CHECK (length(title) BETWEEN 1 AND 96),
  days INTEGER NOT NULL DEFAULT 30 CHECK (days BETWEEN 1 AND 365),
  item_limit INTEGER NOT NULL DEFAULT 5 CHECK (item_limit BETWEEN 1 AND 20),
  sort_order INTEGER NOT NULL DEFAULT 0 CHECK (sort_order BETWEEN 0 AND 1000),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE (profile_id, id),
  FOREIGN KEY (profile_id, dashboard_id) REFERENCES dashboards(profile_id, id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_dashboard_widgets_dashboard_order
  ON dashboard_widgets(profile_id, dashboard_id, sort_order);
