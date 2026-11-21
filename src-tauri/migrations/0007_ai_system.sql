-- Bergag migration 0007: optional, explicitly configured AI providers and local usage history.
-- Credentials are never stored here; only non-secret provider configuration is persisted.
-- All statements are executed inside a transaction by the migration runner.

CREATE TABLE IF NOT EXISTS ai_providers (
  id TEXT PRIMARY KEY,
  profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
  name TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 64),
  name_normalized TEXT NOT NULL CHECK (length(name_normalized) BETWEEN 1 AND 64),
  kind TEXT NOT NULL CHECK (kind IN ('openai', 'anthropic', 'gemini', 'ollama', 'openai_compatible')),
  endpoint TEXT NOT NULL CHECK (length(endpoint) BETWEEN 1 AND 512),
  model TEXT NOT NULL CHECK (length(model) BETWEEN 1 AND 128),
  enabled INTEGER NOT NULL DEFAULT 0 CHECK (enabled IN (0, 1)),
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  UNIQUE (profile_id, id),
  UNIQUE (profile_id, name_normalized)
);

CREATE INDEX IF NOT EXISTS idx_ai_providers_profile_enabled
  ON ai_providers(profile_id, enabled);

CREATE TABLE IF NOT EXISTS ai_usage_events (
  id TEXT PRIMARY KEY,
  profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
  provider_id TEXT,
  provider_name TEXT NOT NULL CHECK (length(provider_name) BETWEEN 1 AND 64),
  provider_kind TEXT NOT NULL CHECK (provider_kind IN ('openai', 'anthropic', 'gemini', 'ollama', 'openai_compatible')),
  feature TEXT NOT NULL CHECK (feature IN ('connection_test', 'summarize', 'suggest_labels', 'ask')),
  success INTEGER NOT NULL CHECK (success IN (0, 1)),
  input_chars INTEGER NOT NULL DEFAULT 0 CHECK (input_chars >= 0),
  output_chars INTEGER NOT NULL DEFAULT 0 CHECK (output_chars >= 0),
  prompt_tokens INTEGER NOT NULL DEFAULT 0 CHECK (prompt_tokens >= 0),
  completion_tokens INTEGER NOT NULL DEFAULT 0 CHECK (completion_tokens >= 0),
  latency_ms INTEGER NOT NULL DEFAULT 0 CHECK (latency_ms >= 0),
  error TEXT,
  created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
  FOREIGN KEY (profile_id, provider_id) REFERENCES ai_providers(profile_id, id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_ai_usage_profile_time
  ON ai_usage_events(profile_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_ai_usage_profile_feature
  ON ai_usage_events(profile_id, feature, created_at DESC);
