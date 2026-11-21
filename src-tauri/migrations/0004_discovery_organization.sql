-- Bergag migration 0004: local discovery directory, labels, feed trust, rules, smart feeds, and rule history.
-- All statements are executed inside a transaction by the migration runner.

ALTER TABLE feeds ADD COLUMN category TEXT;
ALTER TABLE feeds ADD COLUMN trust_level TEXT NOT NULL DEFAULT 'untrusted'
  CHECK (trust_level IN ('untrusted', 'low', 'medium', 'high'));

CREATE INDEX IF NOT EXISTS idx_feeds_profile_trust
  ON feeds(profile_id, trust_level);

CREATE TABLE IF NOT EXISTS discovery_sources (
  id TEXT PRIMARY KEY,
  profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
  feed_url TEXT NOT NULL CHECK (length(feed_url) BETWEEN 1 AND 2048),
  feed_url_normalized TEXT NOT NULL CHECK (length(feed_url_normalized) BETWEEN 1 AND 2048),
  site_url TEXT,
  title TEXT NOT NULL CHECK (length(title) BETWEEN 1 AND 256),
  category TEXT,
  description TEXT,
  trust_level TEXT NOT NULL DEFAULT 'untrusted'
    CHECK (trust_level IN ('untrusted', 'low', 'medium', 'high')),
  subscribed_feed_id TEXT REFERENCES feeds(id) ON DELETE SET NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE (profile_id, feed_url_normalized),
  UNIQUE (profile_id, id)
);

CREATE INDEX IF NOT EXISTS idx_discovery_sources_profile_title
  ON discovery_sources(profile_id, title COLLATE NOCASE);
CREATE INDEX IF NOT EXISTS idx_discovery_sources_profile_category
  ON discovery_sources(profile_id, category COLLATE NOCASE);

CREATE TABLE IF NOT EXISTS labels (
  id TEXT PRIMARY KEY,
  profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
  kind TEXT NOT NULL CHECK (kind IN ('tag', 'topic')),
  name TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 64),
  name_normalized TEXT NOT NULL CHECK (length(name_normalized) BETWEEN 1 AND 64),
  color TEXT,
  description TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE (profile_id, kind, name_normalized),
  UNIQUE (profile_id, id)
);

CREATE INDEX IF NOT EXISTS idx_labels_profile_kind_name
  ON labels(profile_id, kind, name_normalized);

CREATE TABLE IF NOT EXISTS label_assignments (
  profile_id TEXT NOT NULL,
  label_id TEXT NOT NULL,
  item_id TEXT NOT NULL,
  created_at TEXT NOT NULL,
  PRIMARY KEY (label_id, item_id),
  FOREIGN KEY (profile_id, label_id) REFERENCES labels(profile_id, id) ON DELETE CASCADE,
  FOREIGN KEY (profile_id, item_id) REFERENCES feed_items(profile_id, id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_label_assignments_profile_item
  ON label_assignments(profile_id, item_id);
CREATE INDEX IF NOT EXISTS idx_label_assignments_profile_label
  ON label_assignments(profile_id, label_id);

CREATE TABLE IF NOT EXISTS rules (
  id TEXT PRIMARY KEY,
  profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
  name TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 80),
  name_normalized TEXT NOT NULL CHECK (length(name_normalized) BETWEEN 1 AND 80),
  description TEXT,
  rule_json TEXT NOT NULL CHECK (length(rule_json) BETWEEN 2 AND 20000),
  enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
  is_smart_feed INTEGER NOT NULL DEFAULT 0 CHECK (is_smart_feed IN (0, 1)),
  last_run_at TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE (profile_id, name_normalized),
  UNIQUE (profile_id, id)
);

CREATE INDEX IF NOT EXISTS idx_rules_profile_kind
  ON rules(profile_id, is_smart_feed, enabled);
CREATE INDEX IF NOT EXISTS idx_rules_profile_name
  ON rules(profile_id, name_normalized);

CREATE TABLE IF NOT EXISTS rule_runs (
  id TEXT PRIMARY KEY,
  profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
  rule_id TEXT NOT NULL,
  started_at TEXT NOT NULL,
  finished_at TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('success', 'error')),
  matched_count INTEGER NOT NULL DEFAULT 0 CHECK (matched_count >= 0),
  changed_count INTEGER NOT NULL DEFAULT 0 CHECK (changed_count >= 0),
  message TEXT,
  FOREIGN KEY (profile_id, rule_id) REFERENCES rules(profile_id, id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_rule_runs_profile_rule_started
  ON rule_runs(profile_id, rule_id, started_at DESC);
