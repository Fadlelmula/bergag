-- Bergag migration 0001: foundation schema.
-- All statements are executed inside a transaction by the migration runner.

CREATE TABLE IF NOT EXISTS schema_migrations (
  version INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE IF NOT EXISTS app_settings (
  key TEXT PRIMARY KEY,
  value_json TEXT NOT NULL CHECK (json_valid(value_json)),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE IF NOT EXISTS profiles (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 64),
  name_normalized TEXT NOT NULL CHECK (length(name_normalized) BETWEEN 1 AND 64),
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  is_locked INTEGER NOT NULL DEFAULT 0 CHECK (is_locked IN (0, 1)),
  lock_hint TEXT CHECK (lock_hint IS NULL OR length(lock_hint) <= 128)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_profiles_name_normalized
  ON profiles(name_normalized);

CREATE TABLE IF NOT EXISTS profile_settings (
  profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
  key TEXT NOT NULL CHECK (length(key) BETWEEN 1 AND 64),
  value_json TEXT NOT NULL CHECK (json_valid(value_json)),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  PRIMARY KEY (profile_id, key)
);

CREATE INDEX IF NOT EXISTS idx_profile_settings_profile_key
  ON profile_settings(profile_id, key);

CREATE TABLE IF NOT EXISTS layouts (
  id TEXT PRIMARY KEY,
  profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
  name TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 64),
  name_normalized TEXT NOT NULL CHECK (length(name_normalized) BETWEEN 1 AND 64),
  is_active INTEGER NOT NULL DEFAULT 0 CHECK (is_active IN (0, 1)),
  layout_json TEXT NOT NULL CHECK (json_valid(layout_json)),
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  UNIQUE (profile_id, name_normalized)
);

CREATE INDEX IF NOT EXISTS idx_layouts_profile_active
  ON layouts(profile_id, is_active);

CREATE UNIQUE INDEX IF NOT EXISTS idx_layouts_one_active
  ON layouts(profile_id)
  WHERE is_active = 1;
