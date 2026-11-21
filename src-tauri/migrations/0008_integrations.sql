-- Bergag migration 0008: user-added, public-feed-backed integration sources.
-- Integration metadata is profile-scoped and never stores credentials. Items continue to live
-- in the existing feeds/feed_items/article tables through the normal subscription pipeline.

CREATE TABLE IF NOT EXISTS integration_sources (
  id TEXT PRIMARY KEY,
  profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
  kind TEXT NOT NULL CHECK (kind IN (
    'youtube',
    'reddit',
    'github',
    'hacker_news',
    'arxiv',
    'podcast',
    'medium',
    'substack',
    'mastodon',
    'newsletter',
    'website_feed'
  )),
  source_key TEXT NOT NULL CHECK (length(source_key) BETWEEN 1 AND 512),
  display_name TEXT NOT NULL CHECK (length(display_name) BETWEEN 1 AND 256),
  source_url TEXT CHECK (length(source_url) BETWEEN 1 AND 2048),
  feed_url TEXT NOT NULL CHECK (length(feed_url) BETWEEN 1 AND 2048),
  feed_url_normalized TEXT NOT NULL CHECK (length(feed_url_normalized) BETWEEN 1 AND 2048),
  category TEXT CHECK (length(category) BETWEEN 1 AND 64),
  trust_level TEXT NOT NULL DEFAULT 'untrusted'
    CHECK (trust_level IN ('untrusted', 'low', 'medium', 'high')),
  subscribed_feed_id TEXT REFERENCES feeds(id) ON DELETE SET NULL,
  status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'error')),
  last_error TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE (profile_id, id),
  UNIQUE (profile_id, kind, source_key),
  UNIQUE (profile_id, feed_url_normalized)
);

CREATE INDEX IF NOT EXISTS idx_integration_sources_profile_kind
  ON integration_sources(profile_id, kind, display_name COLLATE NOCASE);
CREATE INDEX IF NOT EXISTS idx_integration_sources_profile_feed
  ON integration_sources(profile_id, subscribed_feed_id);
CREATE INDEX IF NOT EXISTS idx_integration_sources_profile_status
  ON integration_sources(profile_id, status);
