-- Bergag migration 0003: article content, reading state, annotations, collections, and offline images.
-- All statements are executed inside a transaction by the migration runner.

CREATE UNIQUE INDEX IF NOT EXISTS idx_feed_items_profile_id
  ON feed_items(profile_id, id);

CREATE TABLE IF NOT EXISTS article_sources (
  item_id TEXT PRIMARY KEY,
  profile_id TEXT NOT NULL,
  raw_html TEXT,
  content_text TEXT NOT NULL DEFAULT '',
  safe_html TEXT NOT NULL DEFAULT '',
  word_count INTEGER NOT NULL DEFAULT 0 CHECK (word_count >= 0),
  image_urls TEXT NOT NULL DEFAULT '[]',
  extracted_at TEXT NOT NULL,
  source TEXT NOT NULL DEFAULT 'feed' CHECK (source IN ('feed', 'manual')),
  FOREIGN KEY (profile_id, item_id) REFERENCES feed_items(profile_id, id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_article_sources_profile_item
  ON article_sources(profile_id, item_id);

CREATE TABLE IF NOT EXISTS article_states (
  item_id TEXT PRIMARY KEY,
  profile_id TEXT NOT NULL,
  is_read INTEGER NOT NULL DEFAULT 0 CHECK (is_read IN (0, 1)),
  is_saved INTEGER NOT NULL DEFAULT 0 CHECK (is_saved IN (0, 1)),
  is_favorite INTEGER NOT NULL DEFAULT 0 CHECK (is_favorite IN (0, 1)),
  is_archived INTEGER NOT NULL DEFAULT 0 CHECK (is_archived IN (0, 1)),
  is_queued INTEGER NOT NULL DEFAULT 0 CHECK (is_queued IN (0, 1)),
  queue_position INTEGER,
  read_count INTEGER NOT NULL DEFAULT 0 CHECK (read_count >= 0),
  last_read_at TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (profile_id, item_id) REFERENCES feed_items(profile_id, id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_article_states_profile_read
  ON article_states(profile_id, is_read);
CREATE INDEX IF NOT EXISTS idx_article_states_profile_saved
  ON article_states(profile_id, is_saved);
CREATE INDEX IF NOT EXISTS idx_article_states_profile_favorite
  ON article_states(profile_id, is_favorite);
CREATE INDEX IF NOT EXISTS idx_article_states_profile_archived
  ON article_states(profile_id, is_archived);
CREATE INDEX IF NOT EXISTS idx_article_states_profile_queue
  ON article_states(profile_id, is_queued, queue_position);

CREATE TABLE IF NOT EXISTS article_history (
  id TEXT PRIMARY KEY,
  profile_id TEXT NOT NULL,
  item_id TEXT NOT NULL,
  event_type TEXT NOT NULL CHECK (event_type IN ('opened', 'read', 'unread')),
  occurred_at TEXT NOT NULL,
  FOREIGN KEY (profile_id, item_id) REFERENCES feed_items(profile_id, id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_article_history_profile_time
  ON article_history(profile_id, occurred_at DESC);
CREATE INDEX IF NOT EXISTS idx_article_history_item_time
  ON article_history(item_id, occurred_at DESC);

CREATE TABLE IF NOT EXISTS article_notes (
  id TEXT PRIMARY KEY,
  profile_id TEXT NOT NULL,
  item_id TEXT NOT NULL,
  body TEXT NOT NULL CHECK (length(body) BETWEEN 1 AND 20000),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (profile_id, item_id) REFERENCES feed_items(profile_id, id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_article_notes_item_created
  ON article_notes(item_id, created_at DESC);

CREATE TABLE IF NOT EXISTS article_highlights (
  id TEXT PRIMARY KEY,
  profile_id TEXT NOT NULL,
  item_id TEXT NOT NULL,
  text TEXT NOT NULL CHECK (length(text) BETWEEN 1 AND 2000),
  start_offset INTEGER NOT NULL CHECK (start_offset >= 0),
  end_offset INTEGER NOT NULL CHECK (end_offset >= start_offset),
  created_at TEXT NOT NULL,
  FOREIGN KEY (profile_id, item_id) REFERENCES feed_items(profile_id, id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_article_highlights_item_created
  ON article_highlights(item_id, created_at DESC);

CREATE TABLE IF NOT EXISTS collections (
  id TEXT PRIMARY KEY,
  profile_id TEXT NOT NULL,
  name TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 64),
  name_normalized TEXT NOT NULL CHECK (length(name_normalized) BETWEEN 1 AND 64),
  description TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE (profile_id, name_normalized),
  UNIQUE (profile_id, id)
);

CREATE INDEX IF NOT EXISTS idx_collections_profile_name
  ON collections(profile_id, name_normalized);

CREATE TABLE IF NOT EXISTS collection_items (
  collection_id TEXT NOT NULL,
  item_id TEXT NOT NULL,
  profile_id TEXT NOT NULL,
  added_at TEXT NOT NULL,
  PRIMARY KEY (collection_id, item_id),
  FOREIGN KEY (profile_id, collection_id) REFERENCES collections(profile_id, id) ON DELETE CASCADE,
  FOREIGN KEY (profile_id, item_id) REFERENCES feed_items(profile_id, id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_collection_items_profile_collection
  ON collection_items(profile_id, collection_id);

CREATE TABLE IF NOT EXISTS article_images (
  item_id TEXT NOT NULL,
  profile_id TEXT NOT NULL,
  url_hash TEXT NOT NULL,
  url TEXT NOT NULL,
  mime TEXT NOT NULL,
  data BLOB NOT NULL,
  created_at TEXT NOT NULL,
  PRIMARY KEY (item_id, url_hash),
  FOREIGN KEY (profile_id, item_id) REFERENCES feed_items(profile_id, id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_article_images_profile_item
  ON article_images(profile_id, item_id);

-- Backfill articles already stored by Milestone 2.
INSERT INTO article_sources (
  item_id, profile_id, raw_html, content_text, safe_html, word_count,
  image_urls, extracted_at, source
)
SELECT
  i.id,
  i.profile_id,
  NULL,
  COALESCE(i.summary, ''),
  REPLACE(REPLACE(REPLACE(REPLACE(COALESCE(i.summary, ''), '&', '&amp;'), '<', '&lt;'), '>', '&gt;'), '"', '&quot;'),
  CASE
    WHEN TRIM(COALESCE(i.summary, '')) = '' THEN 0
    ELSE LENGTH(TRIM(COALESCE(i.summary, ''))) - LENGTH(REPLACE(TRIM(COALESCE(i.summary, '')), ' ', '')) + 1
  END,
  '[]',
  i.discovered_at,
  'feed'
FROM feed_items i
WHERE NOT EXISTS (SELECT 1 FROM article_sources s WHERE s.item_id = i.id);

INSERT INTO article_states (
  item_id, profile_id, is_read, is_saved, is_favorite, is_archived,
  is_queued, queue_position, read_count, last_read_at, created_at, updated_at
)
SELECT i.id, i.profile_id, 0, 0, 0, 0, 0, NULL, 0, NULL, i.discovered_at, i.discovered_at
FROM feed_items i
WHERE NOT EXISTS (SELECT 1 FROM article_states s WHERE s.item_id = i.id);
