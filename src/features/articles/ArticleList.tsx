import {
  articleEmptyMessage,
  articleModeFromNav,
  articleModeLabel,
  articleStateBadges,
  ARTICLE_MODES,
  errorMessage,
  formatArticleDate,
  isArticleView,
} from "../../application/articleService";
import { queryKeys } from "../../application/queryKeys";
import { IconButton } from "../../components/ui/IconButton";
import type {
  ArticleListItem,
  ArticleMode,
  ArticlePresentation,
  ArticleStatePatch,
  ReadingSettings,
} from "../../domain/models";
import type { NavItemId } from "../../domain/navigation";
import { ARTICLE_PRESENTATIONS, normalizeReading, SETTINGS_KEYS } from "../../domain/settings";
import { desktop } from "../../infrastructure/tauri/commands";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { AlignLeft, LayoutGrid, List, Settings } from "lucide-react";
import { useEffect, useMemo, useState } from "react";

interface ArticleListProps {
  profileId: string;
  activeView: NavItemId;
  selectedArticleId: string | null;
  onSelectArticle: (id: string) => void;
  onOpenSettings: () => void;
}

const PRESENTATION_ICONS = {
  list: List,
  magazine: LayoutGrid,
  minimal: AlignLeft,
} as const;

function isTypingTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  return (
    target.isContentEditable ||
    target.tagName === "INPUT" ||
    target.tagName === "TEXTAREA" ||
    target.tagName === "SELECT"
  );
}

export function ArticleList({
  profileId,
  activeView,
  selectedArticleId,
  onSelectArticle,
  onOpenSettings,
}: ArticleListProps) {
  const queryClient = useQueryClient();
  const [modeOverride, setModeOverride] = useState<ArticleMode | null>(null);
  const [collectionId, setCollectionId] = useState<string | null>(null);
  const [selectedIds, setSelectedIds] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);

  const mode = modeOverride ?? (isArticleView(activeView) ? articleModeFromNav(activeView) : "all");

  const readingQuery = useQuery({
    queryKey: queryKeys.settings(profileId, SETTINGS_KEYS.reading),
    queryFn: () => desktop.getProfileSetting<ReadingSettings>(SETTINGS_KEYS.reading),
    enabled: Boolean(profileId),
  });
  const reading = normalizeReading(readingQuery.data);

  const collections = useQuery({
    queryKey: queryKeys.articleCollections(profileId),
    queryFn: desktop.listCollections,
    enabled: Boolean(profileId),
  });

  const effectiveCollectionId =
    mode === "collection" ? (collectionId ?? collections.data?.[0]?.id ?? null) : null;

  const articles = useQuery({
    queryKey: queryKeys.articleList(profileId, mode, effectiveCollectionId),
    queryFn: () => desktop.listArticles(mode, effectiveCollectionId, 200),
    enabled: Boolean(profileId) && (mode !== "collection" || Boolean(effectiveCollectionId)),
  });

  const invalidateArticles = async () => {
    await queryClient.invalidateQueries({ queryKey: queryKeys.articles(profileId) });
  };

  const stateMutation = useMutation({
    mutationFn: ({ id, patch }: { id: string; patch: ArticleStatePatch }) =>
      desktop.setArticleState(id, patch),
    onSuccess: () => {
      setSelectedIds([]);
      void invalidateArticles();
    },
    onError: (mutationError) => setError(errorMessage(mutationError)),
  });

  const bulkMutation = useMutation({
    mutationFn: (patch: ArticleStatePatch) => desktop.bulkUpdateArticles(selectedIds, patch),
    onSuccess: () => {
      setSelectedIds([]);
      void invalidateArticles();
    },
    onError: (mutationError) => setError(errorMessage(mutationError)),
  });

  const presentationMutation = useMutation({
    mutationFn: (presentation: ArticlePresentation) =>
      desktop.setProfileSetting(SETTINGS_KEYS.reading, { ...reading, presentation }),
    onSuccess: () =>
      queryClient.invalidateQueries({
        queryKey: queryKeys.settings(profileId, SETTINGS_KEYS.reading),
      }),
    onError: (mutationError) => setError(errorMessage(mutationError)),
  });

  const rows = useMemo(() => articles.data ?? [], [articles.data]);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.ctrlKey || event.metaKey || event.altKey || isTypingTarget(event.target)) return;
      const key = event.key.toLowerCase();
      if (key !== "j" && key !== "k" && key !== "enter") return;
      if (rows.length === 0) return;
      const eventTarget = event.target;
      if (
        key === "enter" &&
        eventTarget instanceof HTMLElement &&
        (eventTarget.tagName === "BUTTON" || eventTarget.tagName === "A")
      ) {
        return;
      }
      event.preventDefault();
      const current = rows.findIndex((item) => item.id === selectedArticleId);
      const next = key === "j" ? Math.min(current + 1, rows.length - 1) : Math.max(current - 1, 0);
      const target = rows[current === -1 ? 0 : key === "enter" ? current : next];
      if (target) onSelectArticle(target.id);
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [onSelectArticle, rows, selectedArticleId]);

  const toggleSelection = (id: string) => {
    setSelectedIds((current) =>
      current.includes(id) ? current.filter((item) => item !== id) : [...current, id],
    );
  };

  const allSelected = rows.length > 0 && selectedIds.length === rows.length;

  return (
    <div className="bergag-article-list" data-testid="article-list">
      <div className="bergag-article-toolbar">
        <select
          className="bergag-select"
          aria-label="Article list mode"
          value={mode}
          onChange={(event) => {
            setModeOverride(event.target.value as ArticleMode);
            setSelectedIds([]);
          }}
        >
          {ARTICLE_MODES.map((item) => (
            <option key={item} value={item}>
              {articleModeLabel(item)}
            </option>
          ))}
        </select>
        {mode === "collection" ? (
          <select
            className="bergag-select"
            aria-label="Collection"
            value={collectionId ?? ""}
            onChange={(event) => setCollectionId(event.target.value || null)}
          >
            <option value="">Choose collection</option>
            {collections.data?.map((collection) => (
              <option key={collection.id} value={collection.id}>
                {collection.name}
              </option>
            ))}
          </select>
        ) : null}
        <div className="bergag-inline-actions" aria-label="Article presentation">
          {ARTICLE_PRESENTATIONS.map((presentation) => {
            const Icon = PRESENTATION_ICONS[presentation];
            return (
              <IconButton
                key={presentation}
                label={`${presentation[0]?.toUpperCase() ?? ""}${presentation.slice(1)} layout`}
                icon={<Icon size={14} aria-hidden="true" />}
                className={
                  reading.presentation === presentation ? "bergag-icon-button--active" : undefined
                }
                disabled={presentationMutation.isPending}
                onClick={() => presentationMutation.mutate(presentation)}
              />
            );
          })}
        </div>
        <IconButton
          label="Reading settings"
          icon={<Settings size={14} aria-hidden="true" />}
          onClick={onOpenSettings}
        />
      </div>

      {selectedIds.length > 0 ? (
        <div className="bergag-bulk-bar">
          <span>{selectedIds.length} selected</span>
          <button
            type="button"
            className="bergag-link-button"
            disabled={bulkMutation.isPending}
            onClick={() => bulkMutation.mutate({ isRead: true })}
          >
            Mark read
          </button>
          <button
            type="button"
            className="bergag-link-button"
            disabled={bulkMutation.isPending}
            onClick={() => bulkMutation.mutate({ isSaved: true })}
          >
            Save
          </button>
          <button
            type="button"
            className="bergag-link-button"
            disabled={bulkMutation.isPending}
            onClick={() => bulkMutation.mutate({ isFavorite: true })}
          >
            Favorite
          </button>
          <button
            type="button"
            className="bergag-link-button"
            disabled={bulkMutation.isPending}
            onClick={() => bulkMutation.mutate({ isQueued: true })}
          >
            Queue
          </button>
          <button
            type="button"
            className="bergag-link-button"
            disabled={bulkMutation.isPending}
            onClick={() => bulkMutation.mutate({ isArchived: true })}
          >
            Archive
          </button>
          <button type="button" className="bergag-link-button" onClick={() => setSelectedIds([])}>
            Clear
          </button>
        </div>
      ) : null}

      <div className="bergag-article-list__header">
        <span>{articleModeLabel(mode)}</span>
        <span className="bergag-article-list__count">
          {articles.isLoading ? "loading…" : `${rows.length} shown`}
        </span>
      </div>

      {error ? (
        <div className="bergag-error" role="alert">
          {error}
        </div>
      ) : null}
      {articles.isLoading ? <div className="bergag-loading">Loading articles…</div> : null}
      {articles.error ? (
        <div className="bergag-error" role="alert">
          {errorMessage(articles.error)}
        </div>
      ) : null}
      {!articles.isLoading && rows.length === 0 ? (
        <div className="bergag-empty-state">
          <h2 className="bergag-empty-state__title">Nothing here yet</h2>
          <p className="bergag-empty-state__text">{articleEmptyMessage(mode)}</p>
        </div>
      ) : null}

      <div className="bergag-article-list__items">
        {rows.length > 0 ? (
          <label className="bergag-select-all">
            <input
              type="checkbox"
              checked={allSelected}
              onChange={() => setSelectedIds(allSelected ? [] : rows.map((article) => article.id))}
            />
            Select all shown
          </label>
        ) : null}
        {rows.map((article: ArticleListItem) => {
          const badges = articleStateBadges(article.state);
          const isSelected = selectedArticleId === article.id;
          return (
            <div
              key={article.id}
              className={`bergag-article-row bergag-article-row--${reading.presentation} ${isSelected ? "bergag-article-row--active" : ""}`}
            >
              <input
                type="checkbox"
                aria-label={`Select ${article.title}`}
                checked={selectedIds.includes(article.id)}
                onChange={() => toggleSelection(article.id)}
              />
              <button
                type="button"
                className="bergag-article-row__main"
                aria-current={isSelected ? "true" : undefined}
                onClick={() => onSelectArticle(article.id)}
              >
                <span className="bergag-article-row__title">{article.title}</span>
                <span className="bergag-article-row__meta">
                  {article.feed_title} ·{" "}
                  {formatArticleDate(article.published_at ?? article.discovered_at)}
                  {article.word_count > 0 ? ` · ${article.word_count} words` : ""}
                </span>
                {reading.presentation !== "minimal" && article.summary ? (
                  <span className="bergag-article-row__summary">{article.summary}</span>
                ) : null}
                {badges.length > 0 ? (
                  <span className="bergag-article-row__badges">
                    {badges.map((badge) => (
                      <span className="bergag-badge" key={badge}>
                        {badge}
                      </span>
                    ))}
                  </span>
                ) : null}
              </button>
            </div>
          );
        })}
      </div>
      {stateMutation.isPending || bulkMutation.isPending ? (
        <div className="bergag-article-list__busy">Updating…</div>
      ) : null}
    </div>
  );
}
