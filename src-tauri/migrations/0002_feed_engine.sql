-- Bergag migration 0002: feed subscriptions, folders, items, and refresh history.
-- All statements are executed inside a transaction by the migration runner.

CREATE TABLE IF NOT EXISTS feed_folders (
  id TEXT PRIMARY KEY,
  profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
  name TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 64),
  name_normalized TEXT NOT NULL CHECK (length(name_normalized) BETWEEN 1 AND 64),
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  UNIQUE (profile_id, name_normalized),
  UNIQUE (profile_id, id)
);

CREATE INDEX IF NOT EXISTS idx_feed_folders_profile_name
  ON feed_folders(profile_id, name_normalized);

CREATE TABLE IF NOT EXISTS feeds (
  id TEXT PRIMARY KEY,
  profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
  folder_id TEXT,
  title TEXT NOT NULL CHECK (length(title) BETWEEN 1 AND 256),
  site_url TEXT,
  feed_url TEXT NOT NULL CHECK (length(feed_url) BETWEEN 1 AND 2048),
  feed_url_normalized TEXT NOT NULL CHECK (length(feed_url_normalized) BETWEEN 1 AND 2048),
  feed_type TEXT NOT NULL DEFAULT 'unknown' CHECK (feed_type IN ('rss', 'atom', 'json', 'unknown')),
  description TEXT,
  language TEXT,
  status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'paused')),
  etag TEXT,
  last_modified TEXT,
  last_refreshed_at TEXT,
  last_success_at TEXT,
  next_refresh_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  refresh_interval_minutes INTEGER NOT NULL DEFAULT 60 CHECK (refresh_interval_minutes BETWEEN 5 AND 10080),
  consecutive_failures INTEGER NOT NULL DEFAULT 0 CHECK (consecutive_failures >= 0),
  last_error TEXT,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  UNIQUE (profile_id, feed_url_normalized),
  FOREIGN KEY (profile_id, folder_id) REFERENCES feed_folders(profile_id, id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_feeds_profile_folder
  ON feeds(profile_id, folder_id);
CREATE INDEX IF NOT EXISTS idx_feeds_due
  ON feeds(status, next_refresh_at);
CREATE INDEX IF NOT EXISTS idx_feeds_profile_title
  ON feeds(profile_id, title);

CREATE TABLE IF NOT EXISTS feed_items (
  id TEXT PRIMARY KEY,
  profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
  feed_id TEXT NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
  guid TEXT NOT NULL CHECK (length(guid) BETWEEN 1 AND 512),
  title TEXT NOT NULL CHECK (length(title) BETWEEN 1 AND 512),
  url TEXT,
  author TEXT,
  summary TEXT,
  published_at TEXT,
  discovered_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  UNIQUE (feed_id, guid)
);

CREATE INDEX IF NOT EXISTS idx_feed_items_profile_feed_published
  ON feed_items(profile_id, feed_id, published_at DESC);
CREATE INDEX IF NOT EXISTS idx_feed_items_feed_discovered
  ON feed_items(feed_id, discovered_at DESC);

CREATE TABLE IF NOT EXISTS feed_refresh_logs (
  id TEXT PRIMARY KEY,
  profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
  feed_id TEXT NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
  started_at TEXT NOT NULL,
  finished_at TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('success', 'not_modified', 'error', 'skipped')),
  http_status INTEGER CHECK (http_status IS NULL OR http_status BETWEEN 100 AND 599),
  item_count INTEGER NOT NULL DEFAULT 0 CHECK (item_count >= 0),
  new_item_count INTEGER NOT NULL DEFAULT 0 CHECK (new_item_count >= 0),
  duration_ms INTEGER NOT NULL DEFAULT 0 CHECK (duration_ms >= 0),
  message TEXT
);

CREATE INDEX IF NOT EXISTS idx_feed_refresh_logs_feed_started
  ON feed_refresh_logs(feed_id, started_at DESC);
CREATE INDEX IF NOT EXISTS idx_feed_refresh_logs_profile_started
  ON feed_refresh_logs(profile_id, started_at DESC);
