import { errorMessage, formatArticleDate, formatBytes } from "../../application/articleService";
import { queryKeys } from "../../application/queryKeys";
import { Button } from "../../components/ui/Button";
import { IconButton } from "../../components/ui/IconButton";
import type {
  ArticleCollection,
  ArticleDetail,
  ArticleStatePatch,
  Label,
  ReadingSettings,
} from "../../domain/models";
import { normalizeReading, SETTINGS_KEYS } from "../../domain/settings";
import { desktop } from "../../infrastructure/tauri/commands";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import {
  Archive,
  Bookmark,
  Check,
  Download,
  ExternalLink,
  Maximize2,
  Minimize2,
  Pencil,
  Plus,
  RotateCcw,
  Settings,
  Star,
  Trash2,
} from "lucide-react";
import { useEffect, useRef, useState, type CSSProperties } from "react";

interface ArticleReaderProps {
  profileId: string;
  articleId: string | null;
  focusMode: boolean;
  onToggleFocusMode: () => void;
  onOpenSettings: () => void;
  onOpenArticle: (id: string) => void;
  isShortcutBlocked: () => boolean;
}

function isTypingTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  return (
    target.isContentEditable ||
    target.tagName === "INPUT" ||
    target.tagName === "TEXTAREA" ||
    target.tagName === "SELECT"
  );
}

export function ArticleReader({
  profileId,
  articleId,
  focusMode,
  onToggleFocusMode,
  onOpenSettings,
  onOpenArticle,
  isShortcutBlocked,
}: ArticleReaderProps) {
  const queryClient = useQueryClient();
  const contentRef = useRef<HTMLDivElement>(null);
  const openedRef = useRef<string | null>(null);
  const [noteDraft, setNoteDraft] = useState("");
  const [editingNoteId, setEditingNoteId] = useState<string | null>(null);
  const [collectionName, setCollectionName] = useState("");
  const [selectedCollectionId, setSelectedCollectionId] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [extracting, setExtracting] = useState(false);

  const detail = useQuery({
    queryKey: queryKeys.articleDetail(profileId, articleId ?? "none"),
    queryFn: () => desktop.getArticle(articleId ?? ""),
    enabled: Boolean(articleId),
  });
  const readingQuery = useQuery({
    queryKey: queryKeys.settings(profileId, SETTINGS_KEYS.reading),
    queryFn: () => desktop.getProfileSetting<ReadingSettings>(SETTINGS_KEYS.reading),
    enabled: Boolean(profileId),
  });
  const collections = useQuery({
    queryKey: queryKeys.articleCollections(profileId),
    queryFn: desktop.listCollections,
    enabled: Boolean(profileId),
  });
  const allLabels = useQuery({
    queryKey: queryKeys.labels(profileId),
    queryFn: () => desktop.listLabels(null),
    enabled: Boolean(profileId),
  });
  const itemLabels = useQuery({
    queryKey: queryKeys.labelsForItem(profileId, articleId ?? "none"),
    queryFn: () => desktop.labelsForItem(articleId ?? ""),
    enabled: Boolean(articleId),
  });
  const related = useQuery({
    queryKey: queryKeys.relatedArticles(profileId, articleId ?? "none"),
    queryFn: () => desktop.relatedArticles(articleId ?? "", 5),
    enabled: Boolean(articleId),
  });

  const reading = normalizeReading(readingQuery.data);
  const article = detail.data;

  const refresh = async () => {
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: queryKeys.articles(profileId) }),
      queryClient.invalidateQueries({
        queryKey: queryKeys.articleDetail(profileId, articleId ?? "none"),
      }),
      queryClient.invalidateQueries({ queryKey: queryKeys.articleHistory(profileId) }),
      queryClient.invalidateQueries({ queryKey: queryKeys.labels(profileId) }),
      queryClient.invalidateQueries({
        queryKey: queryKeys.labelsForItem(profileId, articleId ?? "none"),
      }),
    ]);
  };

  const run = async (action: () => Promise<unknown>) => {
    setError(null);
    try {
      await action();
      await refresh();
    } catch (actionError) {
      setError(errorMessage(actionError));
    }
  };

  useEffect(() => {
    if (!articleId || !detail.data || openedRef.current === articleId) return;
    openedRef.current = articleId;
    void desktop
      .openArticle(articleId)
      .then(() => refresh())
      .catch((openError: unknown) => setError(errorMessage(openError)));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [articleId, detail.data]);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (
        !article ||
        event.ctrlKey ||
        event.metaKey ||
        event.altKey ||
        isTypingTarget(event.target) ||
        isShortcutBlocked()
      ) {
        return;
      }
      const key = event.key.toLowerCase();
      if (key === "m") {
        event.preventDefault();
        void run(async () => {
          await desktop.setArticleState(article.item.id, {
            isRead: !article.item.state.is_read,
          });
        });
      } else if (key === "s") {
        event.preventDefault();
        void run(async () => {
          await desktop.setArticleState(article.item.id, {
            isSaved: !article.item.state.is_saved,
          });
        });
      } else if (key === "v" && article.item.url) {
        event.preventDefault();
        window.open(article.item.url, "_blank", "noopener,noreferrer");
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [article, isShortcutBlocked]);

  const patchState = (patch: ArticleStatePatch) => {
    if (!article) return;
    void run(async () => {
      await desktop.setArticleState(article.item.id, patch);
    });
  };

  const saveNote = () => {
    if (!article) return;
    const body = noteDraft.trim();
    if (!body) return;
    void run(async () => {
      if (editingNoteId) {
        await desktop.updateArticleNote(editingNoteId, body);
      } else {
        await desktop.addArticleNote(article.item.id, body);
      }
      setNoteDraft("");
      setEditingNoteId(null);
    });
  };

  const saveHighlight = () => {
    if (!article) return;
    const selection = window.getSelection();
    const container = contentRef.current;
    if (
      !selection ||
      selection.isCollapsed ||
      !container ||
      !selection.anchorNode ||
      !selection.focusNode ||
      !container.contains(selection.anchorNode) ||
      !container.contains(selection.focusNode)
    ) {
      return;
    }
    const text = selection.toString().trim();
    if (!text) return;
    if (text.length > 2000) {
      setError("Highlights are limited to 2000 characters.");
      return;
    }
    const foundAt = article.content_text.indexOf(text);
    const start = foundAt >= 0 ? foundAt : 0;
    void run(async () => {
      await desktop.addArticleHighlight(article.item.id, text, start, start + text.length);
      selection.removeAllRanges();
    });
  };

  const extract = () => {
    if (!article?.item.url) return;
    setExtracting(true);
    setError(null);
    void desktop
      .extractArticle(article.item.id)
      .then((result: ArticleDetail) => {
        queryClient.setQueryData(queryKeys.articleDetail(profileId, article.item.id), result);
        return refresh();
      })
      .catch((extractError: unknown) => setError(errorMessage(extractError)))
      .finally(() => setExtracting(false));
  };

  const createCollection = () => {
    const name = collectionName.trim();
    if (!name) return;
    void run(async () => {
      const created = await desktop.createCollection(name, null);
      setCollectionName("");
      setSelectedCollectionId(created.id);
      await queryClient.invalidateQueries({ queryKey: queryKeys.articleCollections(profileId) });
    });
  };

  const addToCollection = () => {
    if (!article || !selectedCollectionId) return;
    void run(async () => {
      await desktop.addToCollection(selectedCollectionId, article.item.id);
    });
  };

  const toggleLabel = (label: Label, assigned: boolean) => {
    if (!article) return;
    void run(async () => {
      if (assigned) {
        await desktop.unassignLabel(label.id, article.item.id);
      } else {
        await desktop.assignLabel(label.id, article.item.id);
      }
    });
  };

  if (!articleId) {
    return (
      <div className="bergag-empty-state" data-testid="article-reader-empty">
        <h2 className="bergag-empty-state__title">Select an article</h2>
        <p className="bergag-empty-state__text">
          Choose an item from the article list. Bergag only shows content stored in this profile.
        </p>
      </div>
    );
  }

  if (detail.isLoading) {
    return <div className="bergag-loading">Opening article…</div>;
  }
  if (detail.error || !article) {
    return (
      <div className="bergag-error" role="alert">
        {errorMessage(detail.error)}
      </div>
    );
  }

  const readerStyle = {
    "--bergag-reader-size": `${reading.fontSize}px`,
    "--bergag-reader-leading": String(reading.lineHeight),
  } as CSSProperties;
  const state = article.item.state;
  const knownCollections = new Set(article.collections.map((item) => item.id));

  return (
    <div className="bergag-reader" style={readerStyle} data-testid="article-reader">
      <div className="bergag-reader__toolbar">
        <div className="bergag-inline-actions">
          <IconButton
            label={state.is_read ? "Mark unread (M)" : "Mark read (M)"}
            icon={<Check size={15} aria-hidden="true" />}
            className={state.is_read ? "bergag-icon-button--active" : undefined}
            onClick={() => patchState({ isRead: !state.is_read })}
          />
          <IconButton
            label={state.is_saved ? "Remove from read later (S)" : "Save for later (S)"}
            icon={<Bookmark size={15} aria-hidden="true" />}
            className={state.is_saved ? "bergag-icon-button--active" : undefined}
            onClick={() => patchState({ isSaved: !state.is_saved })}
          />
          <IconButton
            label={state.is_favorite ? "Remove favorite" : "Add favorite"}
            icon={<Star size={15} aria-hidden="true" />}
            className={state.is_favorite ? "bergag-icon-button--active" : undefined}
            onClick={() => patchState({ isFavorite: !state.is_favorite })}
          />
          <IconButton
            label={state.is_archived ? "Restore from archive" : "Archive article"}
            icon={state.is_archived ? <RotateCcw size={15} /> : <Archive size={15} />}
            onClick={() => patchState({ isArchived: !state.is_archived })}
          />
          <IconButton
            label={state.is_queued ? "Remove from queue" : "Add to reading queue"}
            icon={<Plus size={15} aria-hidden="true" />}
            className={state.is_queued ? "bergag-icon-button--active" : undefined}
            onClick={() => patchState({ isQueued: !state.is_queued })}
          />
        </div>
        <div className="bergag-inline-actions">
          <Button
            variant="ghost"
            size="sm"
            icon={<Download size={14} aria-hidden="true" />}
            disabled={!article.item.url || extracting}
            onClick={extract}
          >
            {extracting ? "Extracting…" : "Extract full text"}
          </Button>
          {article.item.url ? (
            <a
              className="bergag-button bergag-button--ghost bergag-button--sm"
              href={article.item.url}
              target="_blank"
              rel="noreferrer"
              title="Open original website (V)"
            >
              <span className="bergag-button__icon">
                <ExternalLink size={14} aria-hidden="true" />
              </span>
              Original
            </a>
          ) : null}
          <IconButton
            label="Reading settings"
            icon={<Settings size={15} aria-hidden="true" />}
            onClick={onOpenSettings}
          />
          <IconButton
            label={focusMode ? "Exit focus mode (F)" : "Enter focus mode (F)"}
            icon={focusMode ? <Minimize2 size={15} /> : <Maximize2 size={15} />}
            onClick={onToggleFocusMode}
          />
        </div>
      </div>

      {error ? (
        <div className="bergag-reader__error" role="alert">
          {error}
        </div>
      ) : null}

      <div className="bergag-reader__layout">
        <article className="bergag-reader__article">
          <header className="bergag-reader__header">
            <p className="bergag-eyebrow">{article.item.feed_title}</p>
            <h1 className="bergag-title">{article.item.title}</h1>
            <p className="bergag-subtitle">
              {article.item.author ? `${article.item.author} · ` : ""}
              {formatArticleDate(article.item.published_at ?? article.item.discovered_at)}
              {article.item.word_count > 0 ? ` · ${article.item.word_count} words` : ""}
            </p>
            <p className="bergag-reader__source">
              {article.source === "manual"
                ? `Extracted from the original page · ${article.images.length} cached image${article.images.length === 1 ? "" : "s"}`
                : "Feed content stored locally · HTML sanitized before display"}
            </p>
          </header>

          <div
            ref={contentRef}
            className="bergag-reader__content"
            data-testid="reader-content"
            dangerouslySetInnerHTML={{ __html: article.safe_html }}
          />

          <div className="bergag-reader__highlight-action">
            <Button variant="secondary" size="sm" onClick={saveHighlight}>
              Save selected text as highlight
            </Button>
            <span className="bergag-field__hint">
              Selection is read from this article only; highlights stay in this profile.
            </span>
          </div>
        </article>

        <aside className="bergag-reader__aside">
          <section className="bergag-reader__section">
            <h2 className="bergag-section-title">Notes</h2>
            <textarea
              className="bergag-input bergag-textarea"
              aria-label="Article note"
              placeholder="Write a private note…"
              value={noteDraft}
              onChange={(event) => setNoteDraft(event.target.value)}
            />
            <div className="bergag-inline-actions">
              <Button variant="secondary" size="sm" disabled={!noteDraft.trim()} onClick={saveNote}>
                {editingNoteId ? "Update note" : "Add note"}
              </Button>
              {editingNoteId ? (
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => {
                    setEditingNoteId(null);
                    setNoteDraft("");
                  }}
                >
                  Cancel
                </Button>
              ) : null}
            </div>
            {article.notes.length === 0 ? (
              <p className="bergag-field__hint">No notes for this article.</p>
            ) : (
              article.notes.map((note) => (
                <div className="bergag-annotation" key={note.id}>
                  <p>{note.body}</p>
                  <div className="bergag-annotation__footer">
                    <span>{formatArticleDate(note.updated_at)}</span>
                    <div className="bergag-inline-actions">
                      <IconButton
                        label="Edit note"
                        icon={<Pencil size={13} aria-hidden="true" />}
                        onClick={() => {
                          setEditingNoteId(note.id);
                          setNoteDraft(note.body);
                        }}
                      />
                      <IconButton
                        label="Delete note"
                        icon={<Trash2 size={13} aria-hidden="true" />}
                        onClick={() =>
                          void run(async () => {
                            await desktop.deleteArticleNote(note.id);
                          })
                        }
                      />
                    </div>
                  </div>
                </div>
              ))
            )}
          </section>

          <section className="bergag-reader__section">
            <h2 className="bergag-section-title">Highlights</h2>
            {article.highlights.length === 0 ? (
              <p className="bergag-field__hint">Select article text to save a highlight.</p>
            ) : (
              article.highlights.map((highlight) => (
                <div className="bergag-annotation" key={highlight.id}>
                  <blockquote>“{highlight.text}”</blockquote>
                  <div className="bergag-annotation__footer">
                    <span>{formatArticleDate(highlight.created_at)}</span>
                    <IconButton
                      label="Delete highlight"
                      icon={<Trash2 size={13} aria-hidden="true" />}
                      onClick={() =>
                        void run(async () => {
                          await desktop.deleteArticleHighlight(highlight.id);
                        })
                      }
                    />
                  </div>
                </div>
              ))
            )}
          </section>

          <section className="bergag-reader__section">
            <h2 className="bergag-section-title">Collections</h2>
            {article.collections.length > 0 ? (
              <div className="bergag-inline-actions">
                {article.collections.map((collection: ArticleCollection) => (
                  <span
                    className="bergag-badge bergag-badge--accent"
                    key={collection.id}
                    title={collection.description ?? collection.name}
                  >
                    {collection.name}
                    <button
                      type="button"
                      className="bergag-badge__remove"
                      aria-label={`Remove from ${collection.name}`}
                      onClick={() =>
                        void run(async () => {
                          await desktop.removeFromCollection(collection.id, article.item.id);
                        })
                      }
                    >
                      ×
                    </button>
                  </span>
                ))}
              </div>
            ) : (
              <p className="bergag-field__hint">This article is not in a collection.</p>
            )}
            <div className="bergag-field">
              <label className="bergag-field__label" htmlFor="collection-select">
                Add to collection
              </label>
              <select
                id="collection-select"
                className="bergag-select"
                value={selectedCollectionId}
                onChange={(event) => setSelectedCollectionId(event.target.value)}
              >
                <option value="">Choose collection</option>
                {collections.data
                  ?.filter((collection) => !knownCollections.has(collection.id))
                  .map((collection) => (
                    <option key={collection.id} value={collection.id}>
                      {collection.name}
                    </option>
                  ))}
              </select>
              <Button
                variant="secondary"
                size="sm"
                disabled={!selectedCollectionId}
                onClick={addToCollection}
              >
                Add article
              </Button>
            </div>
            <div className="bergag-field">
              <label className="bergag-field__label" htmlFor="new-collection">
                New collection
              </label>
              <input
                id="new-collection"
                className="bergag-input"
                value={collectionName}
                maxLength={64}
                onChange={(event) => setCollectionName(event.target.value)}
                placeholder="Collection name"
              />
              <Button
                variant="ghost"
                size="sm"
                disabled={!collectionName.trim()}
                onClick={createCollection}
              >
                Create collection
              </Button>
            </div>
          </section>

          <section className="bergag-reader__section">
            <h2 className="bergag-section-title">Tags & topics</h2>
            {(itemLabels.data ?? []).length > 0 ? (
              <div className="bergag-inline-actions">
                {(itemLabels.data ?? []).map((label) => (
                  <span className="bergag-badge bergag-badge--accent" key={label.id}>
                    {label.name}
                    <button
                      type="button"
                      className="bergag-badge__remove"
                      aria-label={`Remove ${label.name} from article`}
                      onClick={() => toggleLabel(label, true)}
                    >
                      ×
                    </button>
                  </span>
                ))}
              </div>
            ) : (
              <p className="bergag-field__hint">No labels on this article yet.</p>
            )}
            {(allLabels.data ?? []).length > 0 ? (
              <div className="bergag-label-picker" role="group" aria-label="Assign labels">
                {(allLabels.data ?? []).map((label) => {
                  const assigned = (itemLabels.data ?? []).some((item) => item.id === label.id);
                  return (
                    <button
                      type="button"
                      key={label.id}
                      className={`bergag-label-chip ${assigned ? "bergag-label-chip--active" : ""}`}
                      aria-pressed={assigned}
                      onClick={() => toggleLabel(label, assigned)}
                    >
                      {label.name}
                    </button>
                  );
                })}
              </div>
            ) : (
              <p className="bergag-field__hint">
                Create tags or topics in the Topics view to organize this article.
              </p>
            )}
          </section>

          <section className="bergag-reader__section">
            <h2 className="bergag-section-title">Related articles</h2>
            {related.isLoading ? (
              <p className="bergag-field__hint">Looking for local connections…</p>
            ) : null}
            {related.error ? (
              <p className="bergag-field__hint">{errorMessage(related.error)}</p>
            ) : null}
            {!related.isLoading && (related.data ?? []).length === 0 ? (
              <p className="bergag-field__hint">
                No connections yet. Related articles appear when stories are grouped or labels are
                shared.
              </p>
            ) : (
              (related.data ?? []).map((entry) => (
                <button
                  type="button"
                  className="bergag-related-item"
                  key={entry.item.id}
                  onClick={() => onOpenArticle(entry.item.id)}
                >
                  <span className="bergag-related-item__title">{entry.item.title}</span>
                  <span className="bergag-field__hint">
                    {entry.item.feed_title}
                    {entry.reasons.length > 0 ? ` · ${entry.reasons.join(" · ")}` : ""}
                  </span>
                </button>
              ))
            )}
          </section>

          <section className="bergag-reader__section">
            <h2 className="bergag-section-title">Local cache</h2>
            <p className="bergag-field__hint">
              Article text and cached images remain on this device. Current content source:{" "}
              {article.source}.
            </p>
            {article.images.length > 0 ? (
              <p className="bergag-field__hint">
                {article.images.length} image{article.images.length === 1 ? "" : "s"} embedded as
                local data.
              </p>
            ) : null}
            <OfflineUsage profileId={profileId} />
          </section>
        </aside>
      </div>
    </div>
  );
}

function OfflineUsage({ profileId }: { profileId: string }) {
  const stats = useQuery({
    queryKey: queryKeys.articleCacheStats(profileId),
    queryFn: desktop.articleCacheStats,
  });
  if (!stats.data) return null;
  return (
    <p className="bergag-field__hint">
      Profile cache: {stats.data.articles} articles, {stats.data.images} images,{" "}
      {formatBytes(stats.data.article_bytes + stats.data.image_bytes)}.
    </p>
  );
}
