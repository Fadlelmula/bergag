import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Layers, Play, Save, Search, Trash2 } from "lucide-react";
import { useState } from "react";
import { errorMessage, formatArticleDate } from "../../application/articleService";
import { queryKeys } from "../../application/queryKeys";
import {
  formatStoryBucketDate,
  SEARCH_OPERATOR_HELP,
  SEARCH_SCOPES,
} from "../../application/searchService";
import { Button } from "../../components/ui/Button";
import { IconButton } from "../../components/ui/IconButton";
import type { SearchScope, StoryGroup } from "../../domain/models";
import { desktop } from "../../infrastructure/tauri/commands";

interface SearchViewProps {
  profileId: string;
  onOpenArticle: (id: string) => void;
}

type SearchTab = "search" | "stories";

export function SearchView({ profileId, onOpenArticle }: SearchViewProps) {
  const [tab, setTab] = useState<SearchTab>("search");
  return (
    <div className="bergag-full-view">
      <div className="bergag-full-view__header">
        <div>
          <h2 className="bergag-title">Search & stories</h2>
          <p className="bergag-subtitle">
            Full-text search over the content stored in this profile, plus duplicate coverage
            grouped into local story timelines.
          </p>
        </div>
        <div className="bergag-segmented" role="tablist" aria-label="Search sections">
          <button
            type="button"
            role="tab"
            aria-selected={tab === "search"}
            className={`bergag-segmented__item ${tab === "search" ? "bergag-segmented__item--active" : ""}`}
            onClick={() => setTab("search")}
          >
            <Search size={14} aria-hidden="true" /> Search
          </button>
          <button
            type="button"
            role="tab"
            aria-selected={tab === "stories"}
            className={`bergag-segmented__item ${tab === "stories" ? "bergag-segmented__item--active" : ""}`}
            onClick={() => setTab("stories")}
          >
            <Layers size={14} aria-hidden="true" /> Stories
          </button>
        </div>
      </div>
      <div className="bergag-full-view__body">
        {tab === "search" ? (
          <SearchPanel profileId={profileId} onOpenArticle={onOpenArticle} />
        ) : (
          <StoriesPanel profileId={profileId} onOpenArticle={onOpenArticle} />
        )}
      </div>
    </div>
  );
}

function SearchPanel({
  profileId,
  onOpenArticle,
}: {
  profileId: string;
  onOpenArticle: (id: string) => void;
}) {
  const queryClient = useQueryClient();
  const [queryText, setQueryText] = useState("");
  const [scope, setScope] = useState<SearchScope>("all");
  const [submitted, setSubmitted] = useState<{ query: string; scope: SearchScope } | null>(null);
  const [presetName, setPresetName] = useState("");
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const presets = useQuery({
    queryKey: queryKeys.searchPresets(profileId),
    queryFn: desktop.listSearchPresets,
    enabled: Boolean(profileId),
  });

  const results = useQuery({
    queryKey: queryKeys.search(profileId, submitted?.query ?? "", submitted?.scope ?? "all"),
    queryFn: () => desktop.search(submitted?.query ?? "", submitted?.scope ?? "all", 100),
    enabled: Boolean(profileId && submitted),
  });

  const savePreset = useMutation({
    mutationFn: () => desktop.saveSearchPreset(presetName, queryText, scope),
    onSuccess: async (preset) => {
      setPresetName("");
      setNotice(`Saved “${preset.name}”.`);
      setError(null);
      await queryClient.invalidateQueries({ queryKey: queryKeys.searchPresets(profileId) });
      await queryClient.invalidateQueries({ queryKey: queryKeys.searchStats(profileId) });
    },
    onError: (mutationError) => setError(errorMessage(mutationError)),
  });

  const deletePreset = useMutation({
    mutationFn: (id: string) => desktop.deleteSearchPreset(id),
    onSuccess: async () => {
      setNotice("Saved search deleted.");
      setError(null);
      await queryClient.invalidateQueries({ queryKey: queryKeys.searchPresets(profileId) });
      await queryClient.invalidateQueries({ queryKey: queryKeys.searchStats(profileId) });
    },
    onError: (mutationError) => setError(errorMessage(mutationError)),
  });

  const submit = () => {
    if (!queryText.trim()) {
      setError("Enter a search term or a supported filter.");
      return;
    }
    setError(null);
    setNotice(null);
    setSubmitted({ query: queryText.trim(), scope });
  };

  return (
    <div className="bergag-search-layout">
      <section className="bergag-search-main">
        <form
          className="bergag-search-form"
          onSubmit={(event) => {
            event.preventDefault();
            submit();
          }}
        >
          <input
            className="bergag-input bergag-search-input"
            type="search"
            aria-label="Search query"
            placeholder="Search local articles, e.g. rust is:unread tag:Security"
            value={queryText}
            maxLength={500}
            onChange={(event) => setQueryText(event.target.value)}
          />
          <select
            className="bergag-select"
            aria-label="Search scope"
            value={scope}
            onChange={(event) => setScope(event.target.value as SearchScope)}
          >
            {SEARCH_SCOPES.map((item) => (
              <option key={item.id} value={item.id}>
                {item.label}
              </option>
            ))}
          </select>
          <Button type="submit" variant="primary" icon={<Search size={14} />}>
            Search
          </Button>
        </form>

        <div className="bergag-search-presets">
          <div className="bergag-search-presets__saved">
            {(presets.data ?? []).map((preset) => (
              <span className="bergag-badge bergag-badge--accent" key={preset.id}>
                <button
                  type="button"
                  className="bergag-badge__label"
                  onClick={() => {
                    setQueryText(preset.query);
                    setScope(preset.scope);
                    setSubmitted({ query: preset.query, scope: preset.scope });
                    setError(null);
                    setNotice(null);
                  }}
                >
                  {preset.name}
                </button>
                <button
                  type="button"
                  className="bergag-badge__remove"
                  aria-label={`Delete saved search ${preset.name}`}
                  disabled={deletePreset.isPending}
                  onClick={() => deletePreset.mutate(preset.id)}
                >
                  ×
                </button>
              </span>
            ))}
            {!presets.isLoading && (presets.data ?? []).length === 0 ? (
              <span className="bergag-field__hint">No saved searches yet.</span>
            ) : null}
          </div>
          <div className="bergag-search-presets__save">
            <input
              className="bergag-input"
              aria-label="Saved search name"
              placeholder="Name this search"
              maxLength={64}
              value={presetName}
              onChange={(event) => setPresetName(event.target.value)}
            />
            <Button
              variant="secondary"
              size="sm"
              icon={<Save size={14} />}
              disabled={!presetName.trim() || !queryText.trim() || savePreset.isPending}
              onClick={() => savePreset.mutate()}
            >
              Save search
            </Button>
          </div>
        </div>

        {error ? (
          <div className="bergag-error" role="alert">
            {error}
          </div>
        ) : null}
        {notice ? <div className="bergag-note">{notice}</div> : null}

        <div className="bergag-search-results">
          <div className="bergag-article-list__header">
            <span>{submitted ? `Results for “${submitted.query}”` : "Local library search"}</span>
            <span className="bergag-article-list__count">
              {results.isFetching
                ? "searching…"
                : submitted
                  ? `${results.data?.length ?? 0} found`
                  : ""}
            </span>
          </div>
          {results.isLoading ? (
            <div className="bergag-loading">Searching local content…</div>
          ) : null}
          {results.error ? (
            <div className="bergag-error" role="alert">
              {errorMessage(results.error)}
            </div>
          ) : null}
          {!submitted ? (
            <div className="bergag-empty-state">
              <h2 className="bergag-empty-state__title">Search your stored articles</h2>
              <p className="bergag-empty-state__text">
                Nothing leaves this device. Search covers titles, stored article text, authors, and
                feed titles in this profile.
              </p>
            </div>
          ) : null}
          {submitted && !results.isLoading && (results.data ?? []).length === 0 ? (
            <div className="bergag-empty-state">
              <h2 className="bergag-empty-state__title">No matches</h2>
              <p className="bergag-empty-state__text">
                Try fewer terms, a wider scope, or remove a filter such as is:unread.
              </p>
            </div>
          ) : null}
          {(results.data ?? []).map((result) => (
            <button
              type="button"
              className="bergag-search-result"
              key={result.item.id}
              onClick={() => onOpenArticle(result.item.id)}
            >
              <span className="bergag-article-row__title">{result.item.title}</span>
              <span className="bergag-article-row__meta">
                {result.item.feed_title} ·{" "}
                {formatArticleDate(result.item.published_at ?? result.item.discovered_at)}
                {result.item.author ? ` · ${result.item.author}` : ""}
              </span>
              {result.snippet ? (
                <span className="bergag-article-row__summary">{result.snippet}</span>
              ) : null}
            </button>
          ))}
        </div>
      </section>

      <aside className="bergag-search-help">
        <h3 className="bergag-section-title">Operators</h3>
        <p className="bergag-field__hint">
          Combine words with spaces. Quote phrases. Prefix a term with - to exclude it.
        </p>
        <dl className="bergag-search-help__list">
          {SEARCH_OPERATOR_HELP.map(([operator, description]) => (
            <div key={operator}>
              <dt>
                <code>{operator}</code>
              </dt>
              <dd>{description}</dd>
            </div>
          ))}
        </dl>
        <p className="bergag-field__hint">
          Semantic search is planned for a later milestone. Bergag does not run a hidden model or
          send your library anywhere.
        </p>
      </aside>
    </div>
  );
}

function StoriesPanel({
  profileId,
  onOpenArticle,
}: {
  profileId: string;
  onOpenArticle: (id: string) => void;
}) {
  const queryClient = useQueryClient();
  const [days, setDays] = useState(30);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const timeline = useQuery({
    queryKey: queryKeys.storyTimeline(profileId, days),
    queryFn: () => desktop.storyTimeline(days, 200),
    enabled: Boolean(profileId),
  });

  const detect = useMutation({
    mutationFn: desktop.detectStories,
    onSuccess: async (result) => {
      setNotice(
        `Grouped ${result.items_grouped} article${result.items_grouped === 1 ? "" : "s"} into ${result.groups_created} new story group${result.groups_created === 1 ? "" : "s"}.`,
      );
      setError(null);
      await Promise.all([
        queryClient.invalidateQueries({ queryKey: queryKeys.stories(profileId) }),
        queryClient.invalidateQueries({ queryKey: ["stories", profileId] }),
        queryClient.invalidateQueries({ queryKey: queryKeys.searchStats(profileId) }),
      ]);
    },
    onError: (mutationError) => setError(errorMessage(mutationError)),
  });

  const remove = useMutation({
    mutationFn: (id: string) => desktop.deleteStory(id),
    onSuccess: async () => {
      setNotice("Story group removed. Articles were kept in the library.");
      setError(null);
      await queryClient.invalidateQueries({ queryKey: ["stories", profileId] });
      await queryClient.invalidateQueries({ queryKey: queryKeys.searchStats(profileId) });
    },
    onError: (mutationError) => setError(errorMessage(mutationError)),
  });

  const buckets = timeline.data ?? [];
  const total = buckets.reduce((sum, bucket) => sum + bucket.stories.length, 0);

  return (
    <div className="bergag-stories">
      <div className="bergag-stories__toolbar">
        <div>
          <h3 className="bergag-section-title">Story timeline</h3>
          <p className="bergag-field__hint">
            Duplicate coverage is matched by canonical URL and title similarity. Groups are created
            only when you choose Detect duplicates.
          </p>
        </div>
        <div className="bergag-inline-actions">
          <select
            className="bergag-select"
            aria-label="Timeline range"
            value={days}
            onChange={(event) => setDays(Number(event.target.value))}
          >
            <option value={7}>Last 7 days</option>
            <option value={30}>Last 30 days</option>
            <option value={90}>Last 90 days</option>
            <option value={365}>Last year</option>
          </select>
          <Button
            variant="secondary"
            size="sm"
            icon={<Play size={14} />}
            disabled={detect.isPending}
            onClick={() => detect.mutate()}
          >
            {detect.isPending ? "Detecting…" : "Detect duplicates"}
          </Button>
        </div>
      </div>

      {error ? (
        <div className="bergag-error" role="alert">
          {error}
        </div>
      ) : null}
      {notice ? <div className="bergag-note">{notice}</div> : null}

      {timeline.isLoading ? <div className="bergag-loading">Loading story groups…</div> : null}
      {timeline.error ? (
        <div className="bergag-error" role="alert">
          {errorMessage(timeline.error)}
        </div>
      ) : null}
      {!timeline.isLoading && total === 0 ? (
        <div className="bergag-empty-state">
          <h2 className="bergag-empty-state__title">No story groups yet</h2>
          <p className="bergag-empty-state__text">
            Refresh your feeds, then choose Detect duplicates to group coverage of the same story.
            Existing articles are never deleted or reordered.
          </p>
        </div>
      ) : null}

      {buckets.map((bucket) => (
        <section className="bergag-story-bucket" key={bucket.date}>
          <h4 className="bergag-story-bucket__date">{formatStoryBucketDate(bucket.date)}</h4>
          {bucket.stories.map((story) => (
            <StoryCard
              key={story.id}
              story={story}
              onOpenArticle={onOpenArticle}
              onDelete={() => {
                if (window.confirm("Remove this story group? Articles stay in your library.")) {
                  remove.mutate(story.id);
                }
              }}
            />
          ))}
        </section>
      ))}
    </div>
  );
}

function StoryCard({
  story,
  onOpenArticle,
  onDelete,
}: {
  story: StoryGroup;
  onOpenArticle: (id: string) => void;
  onDelete: () => void;
}) {
  return (
    <article className="bergag-story-card">
      <div className="bergag-story-card__header">
        <div>
          <h5 className="bergag-story-card__title">{story.title}</h5>
          <p className="bergag-field__hint">
            {story.item_count} article{story.item_count === 1 ? "" : "s"} from {story.source_count}{" "}
            source{story.source_count === 1 ? "" : "s"}
            {story.latest_at ? ` · latest ${formatArticleDate(story.latest_at)}` : ""}
          </p>
        </div>
        <IconButton
          label="Remove story group"
          icon={<Trash2 size={14} aria-hidden="true" />}
          onClick={onDelete}
        />
      </div>
      <div className="bergag-story-card__items">
        {story.items.map((item) => (
          <button
            type="button"
            className="bergag-story-card__item"
            key={item.id}
            onClick={() => onOpenArticle(item.id)}
          >
            <span>{item.title}</span>
            <span className="bergag-field__hint">{item.feed_title}</span>
          </button>
        ))}
      </div>
    </article>
  );
}
