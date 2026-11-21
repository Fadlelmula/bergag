import { useQuery } from "@tanstack/react-query";
import { errorMessage, formatArticleDate, formatBytes } from "../../application/articleService";
import {
  countLabel,
  formatCount,
  hasReadingActivity,
  READING_RANGES,
  type ReadingRange,
} from "../../application/analyticsService";
import { formatRelativeTime, healthLabel } from "../../application/feedService";
import { queryKeys } from "../../application/queryKeys";
import type { FeedHealth } from "../../domain/models";
import { desktop } from "../../infrastructure/tauri/commands";
import { Activity, BarChart3, Database, LayoutDashboard, Library } from "lucide-react";
import { useState } from "react";
import { ActivityChart, AnalyticsEmpty, StatCard } from "./AnalyticsBits";
import { DashboardPanel } from "./DashboardPanel";

interface StatisticsViewProps {
  profileId: string;
  onOpenArticle: (id: string) => void;
}

type StatisticsTab = "reading" | "dashboards" | "library" | "feedHealth" | "storage";

const TABS: Array<{ id: StatisticsTab; label: string; icon: typeof BarChart3 }> = [
  { id: "reading", label: "Reading", icon: BarChart3 },
  { id: "dashboards", label: "Dashboards", icon: LayoutDashboard },
  { id: "library", label: "Library", icon: Library },
  { id: "feedHealth", label: "Feed health", icon: Activity },
  { id: "storage", label: "Storage", icon: Database },
];

export function StatisticsView({ profileId, onOpenArticle }: StatisticsViewProps) {
  const [tab, setTab] = useState<StatisticsTab>("reading");
  return (
    <div className="bergag-full-view">
      <div className="bergag-full-view__header">
        <div>
          <h2 className="bergag-title">Statistics</h2>
          <p className="bergag-subtitle">
            Private, on-demand analytics computed from this profile's local SQLite data. Nothing is
            uploaded or inferred from external services.
          </p>
        </div>
        <div className="bergag-segmented" role="tablist" aria-label="Statistics sections">
          {TABS.map((option) => (
            <button
              type="button"
              role="tab"
              aria-selected={tab === option.id}
              key={option.id}
              className={`bergag-segmented__item ${
                tab === option.id ? "bergag-segmented__item--active" : ""
              }`}
              onClick={() => setTab(option.id)}
            >
              <option.icon size={14} aria-hidden="true" /> {option.label}
            </button>
          ))}
        </div>
      </div>
      <div className="bergag-full-view__body">
        {tab === "reading" ? <ReadingPanel profileId={profileId} /> : null}
        {tab === "dashboards" ? (
          <DashboardPanel profileId={profileId} onOpenArticle={onOpenArticle} />
        ) : null}
        {tab === "library" ? <LibraryPanel profileId={profileId} /> : null}
        {tab === "feedHealth" ? <FeedHealthPanel profileId={profileId} /> : null}
        {tab === "storage" ? <StoragePanel profileId={profileId} /> : null}
      </div>
    </div>
  );
}

function ReadingPanel({ profileId }: { profileId: string }) {
  const [days, setDays] = useState<ReadingRange>(30);
  const stats = useQuery({
    queryKey: queryKeys.readingStats(profileId, days),
    queryFn: () => desktop.analyticsReading(days),
    enabled: Boolean(profileId),
  });

  return (
    <div className="bergag-analytics-panel">
      <div className="bergag-analytics-toolbar">
        <div>
          <h3 className="bergag-section-title">Reading activity</h3>
          <p className="bergag-subtitle">
            Open and read events recorded in this profile. No tracking pixels or remote analytics.
          </p>
        </div>
        <div className="bergag-segmented" role="group" aria-label="Reading range">
          {READING_RANGES.map((range) => (
            <button
              type="button"
              key={range}
              aria-pressed={days === range}
              className={`bergag-segmented__item ${
                days === range ? "bergag-segmented__item--active" : ""
              }`}
              onClick={() => setDays(range)}
            >
              {range} days
            </button>
          ))}
        </div>
      </div>
      {stats.isLoading ? <div className="bergag-loading">Loading reading statistics…</div> : null}
      {stats.isError ? (
        <div className="bergag-error" role="alert">
          {errorMessage(stats.error)}
        </div>
      ) : null}
      {stats.data ? (
        <>
          <div className="bergag-card-grid">
            <StatCard
              label="Opened"
              value={formatCount(stats.data.total_opened)}
              detail="All-time local open events"
            />
            <StatCard
              label="Read"
              value={formatCount(stats.data.total_read)}
              detail="All-time local read events"
            />
            <StatCard
              label="Streak"
              value={countLabel(stats.data.streak_days, "day")}
              detail="Consecutive days with a read event"
            />
            <StatCard
              label="Highlights"
              value={formatCount(stats.data.highlights)}
              detail="Saved passages"
            />
            <StatCard
              label="Notes"
              value={formatCount(stats.data.notes)}
              detail="Private article notes"
            />
            <StatCard
              label="Last read"
              value={formatArticleDate(stats.data.last_read_at)}
              detail="From local article state"
            />
          </div>
          <div className="bergag-card bergag-card--padded">
            <h4 className="bergag-title bergag-title--small">Daily events</h4>
            {hasReadingActivity(stats.data) ? (
              <ActivityChart activity={stats.data.activity} />
            ) : (
              <AnalyticsEmpty
                title="No reading events yet"
                text="Open or finish an article; its local history will appear in this chart."
              />
            )}
          </div>
          <div className="bergag-note">
            Totals count all stored events; the chart shows only the selected {days}-day window.
          </div>
        </>
      ) : null}
    </div>
  );
}

function LibraryPanel({ profileId }: { profileId: string }) {
  const stats = useQuery({
    queryKey: queryKeys.libraryStats(profileId),
    queryFn: desktop.analyticsLibrary,
    enabled: Boolean(profileId),
  });

  return (
    <div className="bergag-analytics-panel">
      <h3 className="bergag-section-title">Library composition</h3>
      <p className="bergag-subtitle">Counts include only items stored under this profile.</p>
      {stats.isLoading ? <div className="bergag-loading">Loading library statistics…</div> : null}
      {stats.isError ? (
        <div className="bergag-error" role="alert">
          {errorMessage(stats.error)}
        </div>
      ) : null}
      {stats.data ? (
        <>
          <div className="bergag-card-grid">
            <StatCard label="Articles" value={formatCount(stats.data.articles)} />
            <StatCard label="Feeds" value={formatCount(stats.data.feeds)} />
            <StatCard label="Folders" value={formatCount(stats.data.folders)} />
            <StatCard label="Collections" value={formatCount(stats.data.collections)} />
            <StatCard label="Tags & topics" value={formatCount(stats.data.labels)} />
            <StatCard label="Story groups" value={formatCount(stats.data.stories)} />
            <StatCard label="Saved searches" value={formatCount(stats.data.presets)} />
            <StatCard label="Highlights" value={formatCount(stats.data.highlights)} />
            <StatCard label="Notes" value={formatCount(stats.data.notes)} />
            <StatCard
              label="Stored words"
              value={formatCount(stats.data.words)}
              detail="Extracted article text"
            />
          </div>
          <div className="bergag-card bergag-card--padded">
            <div className="bergag-switch-row">
              <div className="bergag-switch-row__text">
                <span className="bergag-switch-row__label">Oldest discovered article</span>
                <span className="bergag-switch-row__hint">
                  Based on the local discovery timestamp.
                </span>
              </div>
              <span className="bergag-badge">
                {formatArticleDate(stats.data.oldest_article_at)}
              </span>
            </div>
            <div className="bergag-switch-row">
              <div className="bergag-switch-row__text">
                <span className="bergag-switch-row__label">Newest discovered article</span>
                <span className="bergag-switch-row__hint">
                  Based on the local discovery timestamp.
                </span>
              </div>
              <span className="bergag-badge">
                {formatArticleDate(stats.data.newest_article_at)}
              </span>
            </div>
          </div>
        </>
      ) : null}
    </div>
  );
}

function FeedHealthPanel({ profileId }: { profileId: string }) {
  const feeds = useQuery({
    queryKey: queryKeys.feedHealth(profileId, 20),
    queryFn: () => desktop.analyticsFeedHealth(20),
    enabled: Boolean(profileId),
  });

  return (
    <div className="bergag-analytics-panel">
      <h3 className="bergag-section-title">Feed health</h3>
      <p className="bergag-subtitle">
        Refresh state from local feed records and the last 30 days of stored refresh logs.
      </p>
      {feeds.isLoading ? <div className="bergag-loading">Loading feed health…</div> : null}
      {feeds.isError ? (
        <div className="bergag-error" role="alert">
          {errorMessage(feeds.error)}
        </div>
      ) : null}
      {!feeds.isLoading && (feeds.data ?? []).length === 0 ? (
        <AnalyticsEmpty
          title="No subscriptions yet"
          text="Subscribed feeds and their local refresh results will appear here."
        />
      ) : null}
      {feeds.data && feeds.data.length > 0 ? (
        <div className="bergag-health-list">
          {feeds.data.map((feed: FeedHealth) => (
            <div className="bergag-card bergag-card--padded bergag-health-row" key={feed.id}>
              <div className="bergag-health-row__head">
                <h4 className="bergag-title bergag-title--small">{feed.title}</h4>
                <span className={`bergag-badge bergag-badge--${feed.health}`}>
                  {healthLabel(feed.health)}
                </span>
              </div>
              <div className="bergag-health-row__grid">
                <span>Unread: {formatCount(feed.unread)}</span>
                <span>Articles: {formatCount(feed.items)}</span>
                <span>Consecutive failures: {formatCount(feed.consecutive_failures)}</span>
                <span>Failures in 30 days: {formatCount(feed.failures_30d)}</span>
                <span>Last refresh: {formatRelativeTime(feed.last_refreshed_at)}</span>
                <span>Last success: {formatRelativeTime(feed.last_success_at)}</span>
              </div>
              {feed.last_error ? (
                <p className="bergag-stat__detail">Last error: {feed.last_error}</p>
              ) : null}
            </div>
          ))}
        </div>
      ) : null}
    </div>
  );
}

function StoragePanel({ profileId }: { profileId: string }) {
  const stats = useQuery({
    queryKey: queryKeys.storageStats(profileId),
    queryFn: desktop.analyticsStorage,
    enabled: Boolean(profileId),
  });

  return (
    <div className="bergag-analytics-panel">
      <h3 className="bergag-section-title">Local storage</h3>
      <p className="bergag-subtitle">
        Database size is the whole local file; cached content counts are profile-scoped.
      </p>
      {stats.isLoading ? <div className="bergag-loading">Loading storage statistics…</div> : null}
      {stats.isError ? (
        <div className="bergag-error" role="alert">
          {errorMessage(stats.error)}
        </div>
      ) : null}
      {stats.data ? (
        <>
          <div className="bergag-card-grid">
            <StatCard
              label="Database file"
              value={formatBytes(stats.data.database_bytes)}
              detail="Shared local SQLite file"
            />
            <StatCard
              label="Cached images"
              value={countLabel(stats.data.cached_images, "image")}
              detail={`${formatBytes(stats.data.cached_image_bytes)} in this profile`}
            />
            <StatCard
              label="Extracted articles"
              value={formatCount(stats.data.extracted_articles)}
              detail="Locally extracted article content"
            />
            <StatCard
              label="Stored text"
              value={formatCount(stats.data.text_characters)}
              detail="Characters in extracted content"
            />
          </div>
          <div className="bergag-note">
            Storage cleanup is available from Settings → Offline. Statistics never delete data.
          </div>
        </>
      ) : null}
    </div>
  );
}
