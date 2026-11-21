-- Bergag migration 0005: full-text search, saved searches, story groups, and duplicate coverage.
-- All statements are executed inside a transaction by the migration runner.

CREATE VIRTUAL TABLE IF NOT EXISTS search_documents USING fts5(
  profile_id UNINDEXED,
  item_id UNINDEXED,
  title,
  body,
  author,
  feed_title,
  tokenize = 'unicode61 remove_diacritics 2'
);

CREATE TABLE IF NOT EXISTS search_presets (
  id TEXT PRIMARY KEY,
  profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
  name TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 64),
  name_normalized TEXT NOT NULL CHECK (length(name_normalized) BETWEEN 1 AND 64),
  query TEXT NOT NULL CHECK (length(query) BETWEEN 1 AND 500),
  scope TEXT NOT NULL DEFAULT 'all'
    CHECK (scope IN ('all', 'unread', 'saved', 'favorites', 'archive', 'queue')),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE (profile_id, name_normalized),
  UNIQUE (profile_id, id)
);

CREATE INDEX IF NOT EXISTS idx_search_presets_profile_name
  ON search_presets(profile_id, name_normalized);

CREATE TABLE IF NOT EXISTS story_groups (
  id TEXT PRIMARY KEY,
  profile_id TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
  title TEXT NOT NULL CHECK (length(title) BETWEEN 1 AND 512),
  canonical_item_id TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE (profile_id, id),
  FOREIGN KEY (profile_id, canonical_item_id) REFERENCES feed_items(profile_id, id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_story_groups_profile_updated
  ON story_groups(profile_id, updated_at DESC);

CREATE TABLE IF NOT EXISTS story_group_items (
  story_id TEXT NOT NULL,
  item_id TEXT NOT NULL,
  profile_id TEXT NOT NULL,
  similarity REAL NOT NULL DEFAULT 1.0 CHECK (similarity >= 0 AND similarity <= 1),
  added_at TEXT NOT NULL,
  PRIMARY KEY (story_id, item_id),
  FOREIGN KEY (profile_id, story_id) REFERENCES story_groups(profile_id, id) ON DELETE CASCADE,
  FOREIGN KEY (profile_id, item_id) REFERENCES feed_items(profile_id, id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_story_group_items_profile_item
  ON story_group_items(profile_id, item_id);

-- Keep the local FTS index in sync with stored items and extracted article text.
CREATE TRIGGER IF NOT EXISTS feed_items_search_insert
AFTER INSERT ON feed_items
BEGIN
  INSERT INTO search_documents (profile_id, item_id, title, body, author, feed_title)
  VALUES (
    new.profile_id,
    new.id,
    new.title,
    COALESCE(new.summary, ''),
    COALESCE(new.author, ''),
    COALESCE((SELECT title FROM feeds WHERE id = new.feed_id), '')
  );
END;

CREATE TRIGGER IF NOT EXISTS feed_items_search_update
AFTER UPDATE OF title, summary, author, feed_id ON feed_items
BEGIN
  UPDATE search_documents
  SET title = new.title,
      body = COALESCE(
        (SELECT content_text FROM article_sources
          WHERE item_id = new.id AND profile_id = new.profile_id),
        new.summary,
        ''
      ),
      author = COALESCE(new.author, ''),
      feed_title = COALESCE((SELECT title FROM feeds WHERE id = new.feed_id), '')
  WHERE item_id = new.id AND profile_id = new.profile_id;
END;

CREATE TRIGGER IF NOT EXISTS feed_items_search_delete
AFTER DELETE ON feed_items
BEGIN
  DELETE FROM search_documents WHERE item_id = old.id AND profile_id = old.profile_id;
END;

CREATE TRIGGER IF NOT EXISTS article_sources_search_insert
AFTER INSERT ON article_sources
BEGIN
  UPDATE search_documents
  SET body = new.content_text
  WHERE item_id = new.item_id AND profile_id = new.profile_id;
END;

CREATE TRIGGER IF NOT EXISTS article_sources_search_update
AFTER UPDATE OF content_text ON article_sources
BEGIN
  UPDATE search_documents
  SET body = new.content_text
  WHERE item_id = new.item_id AND profile_id = new.profile_id;
END;

CREATE TRIGGER IF NOT EXISTS feeds_search_title_update
AFTER UPDATE OF title ON feeds
BEGIN
  UPDATE search_documents
  SET feed_title = new.title
  WHERE profile_id = new.profile_id
    AND item_id IN (SELECT id FROM feed_items WHERE feed_id = new.id);
END;

-- Index content stored before this migration. Triggers maintain it afterwards.
INSERT INTO search_documents (profile_id, item_id, title, body, author, feed_title)
SELECT
  i.profile_id,
  i.id,
  i.title,
  COALESCE(NULLIF(s.content_text, ''), i.summary, ''),
  COALESCE(i.author, ''),
  f.title
FROM feed_items i
JOIN feeds f ON f.id = i.feed_id AND f.profile_id = i.profile_id
LEFT JOIN article_sources s ON s.item_id = i.id AND s.profile_id = i.profile_id;
