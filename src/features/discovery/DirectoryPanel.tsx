import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ExternalLink, FolderPlus, LoaderCircle, Plus, RefreshCw, Trash2 } from "lucide-react";
import { useMemo, useState } from "react";
import {
  discoveryCategories,
  filterDiscoverySources,
  TRUST_LEVELS,
  trustLabel,
} from "../../application/discoveryService";
import { queryKeys } from "../../application/queryKeys";
import { Button } from "../../components/ui/Button";
import { IconButton } from "../../components/ui/IconButton";
import type { DiscoveryPreviewResult, DiscoverySource, TrustLevel } from "../../domain/models";
import { desktop } from "../../infrastructure/tauri/commands";

interface DirectoryPanelProps {
  profileId: string;
  onSubscribed: () => void;
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export function DirectoryPanel({ profileId, onSubscribed }: DirectoryPanelProps) {
  const queryClient = useQueryClient();
  const sources = useQuery({
    queryKey: queryKeys.discoverySources(profileId),
    queryFn: desktop.listDiscoverySources,
  });
  const folders = useQuery({
    queryKey: queryKeys.feedFolders(profileId),
    queryFn: desktop.listFeedFolders,
  });
  const feeds = useQuery({
    queryKey: queryKeys.feeds(profileId),
    queryFn: desktop.listFeeds,
  });
  const [query, setQuery] = useState("");
  const [categoryFilter, setCategoryFilter] = useState("");
  const [url, setUrl] = useState("");
  const [newCategory, setNewCategory] = useState("");
  const [newTrust, setNewTrust] = useState<TrustLevel>("untrusted");
  const [folderId, setFolderId] = useState("");
  const [preview, setPreview] = useState<DiscoveryPreviewResult | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);

  const categories = useMemo(() => discoveryCategories(sources.data ?? []), [sources.data]);
  const visibleSources = useMemo(
    () => filterDiscoverySources(sources.data ?? [], query, categoryFilter || null),
    [categoryFilter, query, sources.data],
  );
  const subscribedFeedIds = new Set((feeds.data ?? []).map((feed) => feed.id));

  const invalidate = async () => {
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: queryKeys.discoverySources(profileId) }),
      queryClient.invalidateQueries({ queryKey: queryKeys.feeds(profileId) }),
      queryClient.invalidateQueries({ queryKey: queryKeys.feedFolders(profileId) }),
    ]);
  };

  const previewSource = useMutation({
    mutationFn: (value: string) => desktop.previewDiscoverySource(value),
    onSuccess: (result) => {
      setPreview(result);
      setNotice(null);
      setActionError(null);
    },
    onError: (error) => setActionError(errorMessage(error)),
  });

  const addSource = useMutation({
    mutationFn: () => desktop.addDiscoverySource(url, newCategory.trim() || null, newTrust),
    onSuccess: async (result) => {
      setPreview(result);
      setNotice("Saved to the local discovery directory.");
      setActionError(null);
      await invalidate();
    },
    onError: (error) => setActionError(errorMessage(error)),
  });

  const subscribe = useMutation({
    mutationFn: (source: DiscoverySource) =>
      desktop.subscribeDiscoverySource(source.id, folderId || null),
    onSuccess: async (feed) => {
      setNotice(`Subscribed to ${feed.title}.`);
      setActionError(null);
      await invalidate();
      onSubscribed();
    },
    onError: (error) => setActionError(errorMessage(error)),
  });

  const setTrust = useMutation({
    mutationFn: ({ id, trust }: { id: string; trust: TrustLevel }) =>
      desktop.setDiscoveryTrust(id, trust),
    onSuccess: async () => {
      await invalidate();
    },
    onError: (error) => setActionError(errorMessage(error)),
  });

  const remove = useMutation({
    mutationFn: (id: string) => desktop.deleteDiscoverySource(id),
    onSuccess: async () => {
      setPreview(null);
      setNotice("Removed from the local discovery directory. Subscriptions were not changed.");
      await invalidate();
    },
    onError: (error) => setActionError(errorMessage(error)),
  });

  const sourceForPreview = preview?.source ?? null;

  return (
    <div className="bergag-discovery-panel">
      <div className="bergag-discovery-toolbar">
        <div className="bergag-field bergag-field--inline">
          <label className="bergag-field__label" htmlFor="directory-search">
            Search directory
          </label>
          <input
            id="directory-search"
            className="bergag-input"
            value={query}
            placeholder="Title, URL, or category"
            onChange={(event) => setQuery(event.target.value)}
          />
        </div>
        <div className="bergag-field bergag-field--inline">
          <label className="bergag-field__label" htmlFor="directory-category-filter">
            Category
          </label>
          <select
            id="directory-category-filter"
            className="bergag-select"
            value={categoryFilter}
            onChange={(event) => setCategoryFilter(event.target.value)}
          >
            <option value="">All categories</option>
            {categories.map((category) => (
              <option key={category} value={category}>
                {category}
              </option>
            ))}
          </select>
        </div>
        <div className="bergag-field bergag-field--inline">
          <label className="bergag-field__label" htmlFor="directory-folder">
            Subscribe into
          </label>
          <select
            id="directory-folder"
            className="bergag-select"
            value={folderId}
            onChange={(event) => setFolderId(event.target.value)}
          >
            <option value="">No folder</option>
            {(folders.data ?? []).map((folder) => (
              <option key={folder.id} value={folder.id}>
                {folder.name}
              </option>
            ))}
          </select>
        </div>
      </div>

      {notice ? (
        <div className="bergag-feed-notice">
          <span>{notice}</span>
          <Button variant="ghost" size="sm" onClick={() => setNotice(null)}>
            Dismiss
          </Button>
        </div>
      ) : null}
      {actionError ? (
        <div className="bergag-error" role="alert">
          {actionError}
        </div>
      ) : null}

      <div className="bergag-discovery-grid">
        <section className="bergag-discovery-column" aria-label="Discovery directory">
          <div className="bergag-discovery-column__header">
            <h3 className="bergag-title bergag-title--small">Local directory</h3>
            <span className="bergag-feed-toolbar__count">
              {visibleSources.length} of {(sources.data ?? []).length}
            </span>
          </div>

          <div className="bergag-discovery-add">
            <div className="bergag-field">
              <label className="bergag-field__label" htmlFor="discovery-url">
                Feed URL
              </label>
              <input
                id="discovery-url"
                className="bergag-input"
                value={url}
                placeholder="https://example.com/feed.xml"
                onChange={(event) => setUrl(event.target.value)}
              />
              <span className="bergag-field__hint">
                Bergag fetches the feed URL directly, validates public addresses, and never uses a
                third-party discovery service.
              </span>
            </div>
            <div className="bergag-field">
              <label className="bergag-field__label" htmlFor="discovery-category">
                Category
              </label>
              <input
                id="discovery-category"
                className="bergag-input"
                value={newCategory}
                maxLength={64}
                placeholder="Optional category"
                onChange={(event) => setNewCategory(event.target.value)}
              />
            </div>
            <div className="bergag-field">
              <label className="bergag-field__label" htmlFor="discovery-trust">
                Trust level
              </label>
              <select
                id="discovery-trust"
                className="bergag-select"
                value={newTrust}
                onChange={(event) => setNewTrust(event.target.value as TrustLevel)}
              >
                {TRUST_LEVELS.map((level) => (
                  <option key={level.value} value={level.value}>
                    {level.label}
                  </option>
                ))}
              </select>
            </div>
            <div className="bergag-inline-actions">
              <Button
                variant="secondary"
                onClick={() => previewSource.mutate(url)}
                disabled={!url.trim() || previewSource.isPending}
                icon={
                  previewSource.isPending ? <LoaderCircle size={14} /> : <RefreshCw size={14} />
                }
              >
                Preview
              </Button>
              <Button
                variant="primary"
                onClick={() => addSource.mutate()}
                disabled={!url.trim() || addSource.isPending}
                icon={addSource.isPending ? <LoaderCircle size={14} /> : <Plus size={14} />}
              >
                Save to directory
              </Button>
            </div>
          </div>

          <div className="bergag-discovery-list">
            {sources.isLoading ? <div className="bergag-loading">Loading directory…</div> : null}
            {sources.error ? (
              <div className="bergag-error" role="alert">
                {errorMessage(sources.error)}
              </div>
            ) : null}
            {!sources.isLoading && (sources.data ?? []).length === 0 ? (
              <div className="bergag-empty-state">
                <div className="bergag-empty-state__title">No saved sources</div>
                <p className="bergag-empty-state__text">
                  Preview a feed URL above and save it to your private local directory. Nothing is
                  added until you choose Save.
                </p>
              </div>
            ) : null}
            {!sources.isLoading &&
            visibleSources.length === 0 &&
            (sources.data ?? []).length > 0 ? (
              <div className="bergag-empty-state">
                <div className="bergag-empty-state__title">No matches</div>
                <p className="bergag-empty-state__text">
                  Try a different title, URL, or category filter.
                </p>
              </div>
            ) : null}
            {visibleSources.map((source) => {
              const subscribed = Boolean(
                source.subscribed_feed_id && subscribedFeedIds.has(source.subscribed_feed_id),
              );
              return (
                <article className="bergag-discovery-row" key={source.id}>
                  <div className="bergag-discovery-row__main">
                    <div className="bergag-discovery-row__title">{source.title}</div>
                    <div className="bergag-discovery-row__meta">{source.feed_url}</div>
                    <div className="bergag-inline-actions">
                      {source.category ? (
                        <span className="bergag-badge">{source.category}</span>
                      ) : null}
                      <span className="bergag-badge">{trustLabel(source.trust_level)} trust</span>
                      {subscribed ? <span className="bergag-badge">Subscribed</span> : null}
                    </div>
                  </div>
                  <div className="bergag-discovery-row__actions">
                    <select
                      className="bergag-select"
                      aria-label={`Trust level for ${source.title}`}
                      value={source.trust_level}
                      disabled={setTrust.isPending}
                      onChange={(event) =>
                        setTrust.mutate({
                          id: source.id,
                          trust: event.target.value as TrustLevel,
                        })
                      }
                    >
                      {TRUST_LEVELS.map((level) => (
                        <option key={level.value} value={level.value}>
                          {level.label}
                        </option>
                      ))}
                    </select>
                    <Button
                      variant="secondary"
                      size="sm"
                      onClick={() => {
                        setUrl(source.feed_url);
                        setNewCategory(source.category ?? "");
                        setNewTrust(source.trust_level);
                        previewSource.mutate(source.feed_url);
                      }}
                      icon={<RefreshCw size={13} />}
                    >
                      Preview
                    </Button>
                    <Button
                      variant="primary"
                      size="sm"
                      onClick={() => subscribe.mutate(source)}
                      disabled={subscribed || subscribe.isPending}
                      icon={<FolderPlus size={13} />}
                    >
                      {subscribed ? "Subscribed" : "Subscribe"}
                    </Button>
                    <IconButton
                      label={`Remove ${source.title} from directory`}
                      icon={<Trash2 size={13} aria-hidden="true" />}
                      onClick={() => remove.mutate(source.id)}
                    />
                  </div>
                </article>
              );
            })}
          </div>
        </section>

        <section className="bergag-discovery-column" aria-label="Feed preview">
          <div className="bergag-discovery-column__header">
            <h3 className="bergag-title bergag-title--small">Preview</h3>
            {sourceForPreview?.site_url ? (
              <a
                className="bergag-inline-link"
                href={sourceForPreview.site_url}
                target="_blank"
                rel="noreferrer"
              >
                <ExternalLink size={13} aria-hidden="true" /> Website
              </a>
            ) : null}
          </div>
          {!preview ? (
            <div className="bergag-empty-state">
              <div className="bergag-empty-state__title">Nothing previewed</div>
              <p className="bergag-empty-state__text">
                Preview a URL to inspect its title, site, and latest items before saving or
                subscribing.
              </p>
            </div>
          ) : (
            <div className="bergag-preview">
              <div className="bergag-preview__title">{preview.preview.title}</div>
              <div className="bergag-preview__meta">
                {preview.preview.feed_type} · {preview.preview.item_count} item
                {preview.preview.item_count === 1 ? "" : "s"}
                {preview.preview.language ? ` · ${preview.preview.language}` : ""}
              </div>
              {preview.preview.description ? (
                <p className="bergag-preview__description">{preview.preview.description}</p>
              ) : null}
              {sourceForPreview ? (
                <div className="bergag-preview__source">
                  <span>
                    Saved as {sourceForPreview.category ?? "uncategorized"} ·{" "}
                    {trustLabel(sourceForPreview.trust_level)} trust
                  </span>
                  {sourceForPreview.subscribed_feed_id &&
                  subscribedFeedIds.has(sourceForPreview.subscribed_feed_id) ? (
                    <span className="bergag-badge">Subscribed</span>
                  ) : (
                    <Button
                      variant="primary"
                      size="sm"
                      onClick={() => subscribe.mutate(sourceForPreview)}
                      disabled={subscribe.isPending}
                      icon={<FolderPlus size={13} />}
                    >
                      Subscribe
                    </Button>
                  )}
                </div>
              ) : (
                <Button
                  variant="primary"
                  size="sm"
                  onClick={() => addSource.mutate()}
                  disabled={addSource.isPending}
                  icon={<Plus size={13} />}
                >
                  Save to directory
                </Button>
              )}
              <div className="bergag-preview__items">
                {preview.preview.items.slice(0, 8).map((item) => (
                  <div className="bergag-preview__item" key={`${item.guid}-${item.title}`}>
                    <span className="bergag-preview__item-title">{item.title}</span>
                    <span className="bergag-preview__item-meta">
                      {item.published_at ?? item.discovered_at}
                    </span>
                  </div>
                ))}
                {preview.preview.items.length === 0 ? (
                  <div className="bergag-empty-state__text">
                    The feed responded but did not include preview items.
                  </div>
                ) : null}
              </div>
            </div>
          )}
        </section>
      </div>
    </div>
  );
}
