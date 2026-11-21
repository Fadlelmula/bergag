import { useQuery } from "@tanstack/react-query";
import { Activity, ExternalLink } from "lucide-react";
import {
  feedTypeLabel,
  formatRelativeTime,
  formatSchedule,
  healthLabel,
  refreshIntervalLabel,
} from "../../application/feedService";
import { queryKeys } from "../../application/queryKeys";
import type { Feed } from "../../domain/models";
import { desktop } from "../../infrastructure/tauri/commands";

interface FeedDetailProps {
  profileId: string;
  feed: Feed | undefined;
}

export function FeedDetail({ profileId, feed }: FeedDetailProps) {
  const items = useQuery({
    queryKey: queryKeys.feedItems(profileId, feed?.id ?? "none"),
    queryFn: () => desktop.listFeedItems(feed?.id ?? "", 50),
    enabled: Boolean(feed),
  });
  const logs = useQuery({
    queryKey: queryKeys.feedLogs(profileId, feed?.id ?? "none"),
    queryFn: () => desktop.listFeedLogs(feed?.id ?? "", 10),
    enabled: Boolean(feed),
  });

  if (!feed) {
    return (
      <div className="bergag-empty-state">
        <div className="bergag-empty-state__title">Select a feed</div>
        <p className="bergag-empty-state__text">
          Choose a subscription from the list to see its latest discovered items and refresh
          history.
        </p>
      </div>
    );
  }

  return (
    <div className="bergag-feed-detail">
      <div className="bergag-feed-detail__header">
        <div>
          <div className="bergag-eyebrow">Subscription</div>
          <h2 className="bergag-title bergag-title--small">{feed.title}</h2>
          <p className="bergag-subtitle">
            {feed.feed_url} · {feedTypeLabel(feed.feed_type)}
          </p>
        </div>
        <span className={`bergag-badge bergag-badge--${feed.health}`}>
          {healthLabel(feed.health)}
        </span>
      </div>

      <div className="bergag-card-grid">
        <div className="bergag-card bergag-card--padded">
          <div className="bergag-stat__label">Discovered items</div>
          <div className="bergag-stat__value">{feed.article_count}</div>
          <div className="bergag-stat__detail">Stored locally for the reader.</div>
        </div>
        <div className="bergag-card bergag-card--padded">
          <div className="bergag-stat__label">Last refresh</div>
          <div className="bergag-stat__value">{formatRelativeTime(feed.last_refreshed_at)}</div>
          <div className="bergag-stat__detail">
            Next scheduled {formatSchedule(feed.next_refresh_at)}
          </div>
        </div>
        <div className="bergag-card bergag-card--padded">
          <div className="bergag-stat__label">Interval</div>
          <div className="bergag-stat__value">
            {refreshIntervalLabel(feed.refresh_interval_minutes)}
          </div>
          <div className="bergag-stat__detail">
            {feed.consecutive_failures === 0
              ? "No consecutive failures."
              : `${feed.consecutive_failures} consecutive failure${feed.consecutive_failures === 1 ? "" : "s"}; automatic backoff applied.`}
          </div>
        </div>
      </div>

      {feed.last_error ? (
        <div className="bergag-note" style={{ marginBottom: 18 }} role="status">
          {feed.last_error}
        </div>
      ) : null}

      <div className="bergag-section-title">Latest items</div>
      {items.isLoading ? <div className="bergag-loading">Loading items…</div> : null}
      {items.error ? <div className="bergag-error">{String(items.error)}</div> : null}
      {items.data && items.data.length === 0 ? (
        <p className="bergag-field__hint">No items have been discovered from this feed yet.</p>
      ) : null}
      {items.data?.map((item) => (
        <article key={item.id} className="bergag-article-card">
          <div className="bergag-article-card__title">{item.title}</div>
          <div className="bergag-article-card__meta">
            {item.author ? `${item.author} · ` : ""}
            {formatRelativeTime(item.published_at ?? item.discovered_at)}
          </div>
          {item.summary ? <p className="bergag-article-card__summary">{item.summary}</p> : null}
          {item.url ? (
            <a
              className="bergag-article-card__link"
              href={item.url}
              target="_blank"
              rel="noreferrer"
            >
              Open original <ExternalLink size={12} aria-hidden="true" />
            </a>
          ) : null}
        </article>
      ))}

      <div className="bergag-section-title">
        <Activity size={14} aria-hidden="true" /> Refresh history
      </div>
      {logs.isLoading ? <div className="bergag-loading">Loading history…</div> : null}
      {logs.data && logs.data.length === 0 ? (
        <p className="bergag-field__hint">No refreshes recorded yet.</p>
      ) : null}
      {logs.data?.map((log) => (
        <div key={log.id} className="bergag-log-row">
          <span className={`bergag-log-row__status bergag-log-row__status--${log.status}`}>
            {log.status}
          </span>
          <span>{formatRelativeTime(log.started_at)}</span>
          <span>
            {log.item_count} item{log.item_count === 1 ? "" : "s"}
            {log.new_item_count > 0 ? ` · ${log.new_item_count} new` : ""}
          </span>
          <span>{log.duration_ms} ms</span>
          {log.message ? <span className="bergag-log-row__message">{log.message}</span> : null}
        </div>
      ))}
    </div>
  );
}
