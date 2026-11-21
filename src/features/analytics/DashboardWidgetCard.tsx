import { useQuery } from "@tanstack/react-query";
import {
  briefingPeriodForDays,
  widgetKindDescription,
  widgetKindLabel,
} from "../../application/dashboardService";
import { errorMessage, formatArticleDate } from "../../application/articleService";
import { healthLabel } from "../../application/feedService";
import { queryKeys } from "../../application/queryKeys";
import type {
  ArticleListItem,
  Briefing,
  DashboardWidget,
  FeedHealth,
  NamedCount,
  ReadingStats,
  StorageStats,
} from "../../domain/models";
import { desktop } from "../../infrastructure/tauri/commands";
import { ActivityChart, AnalyticsEmpty, CountRows, StatCard } from "./AnalyticsBits";

type WidgetData =
  ReadingStats | StorageStats | Briefing | ArticleListItem[] | NamedCount[] | FeedHealth[] | null;

interface DashboardWidgetCardProps {
  profileId: string;
  widget: DashboardWidget;
  onOpenArticle: (id: string) => void;
}

function scopeLabel(widget: DashboardWidget): string {
  switch (widget.kind) {
    case "briefing":
      return briefingPeriodForDays(widget.days) === "daily" ? "Daily" : "Weekly";
    case "reading_activity":
      return `${widget.days}d`;
    case "feed_health":
      return "30d failures";
    case "top_labels":
      return "All time";
    case "recent_articles":
      return "Latest";
    case "unread_by_feed":
    case "reading_queue":
    case "storage":
      return "Current";
    default:
      return "Local";
  }
}

export function DashboardWidgetCard({
  profileId,
  widget,
  onOpenArticle,
}: DashboardWidgetCardProps) {
  const query = useQuery({
    queryKey: queryKeys.dashboardWidgetData(
      profileId,
      widget.id,
      widget.kind,
      widget.days,
      widget.item_limit,
    ),
    enabled: Boolean(profileId),
    queryFn: async (): Promise<WidgetData> => {
      switch (widget.kind) {
        case "unread_by_feed":
          return desktop.analyticsUnreadFeeds(widget.item_limit);
        case "recent_articles":
          return desktop.analyticsArticles("recent", widget.item_limit);
        case "reading_activity":
          return desktop.analyticsReading(widget.days);
        case "top_labels":
          return desktop.analyticsTopLabels(widget.item_limit);
        case "feed_health":
          return desktop.analyticsFeedHealth(widget.item_limit);
        case "storage":
          return desktop.analyticsStorage();
        case "briefing":
          return desktop.buildBriefing(briefingPeriodForDays(widget.days));
        case "reading_queue":
          return desktop.analyticsArticles("queue", widget.item_limit);
        default:
          return null;
      }
    },
  });

  return (
    <section className="bergag-widget-card" aria-label={widget.title}>
      <header className="bergag-widget-card__header">
        <div>
          <div className="bergag-eyebrow">{widgetKindLabel(widget.kind)}</div>
          <h3 className="bergag-widget-card__title">{widget.title}</h3>
        </div>
        <span className="bergag-badge">{scopeLabel(widget)}</span>
      </header>
      <p className="bergag-widget-card__description">{widgetKindDescription(widget.kind)}</p>
      {query.isLoading ? <div className="bergag-loading">Loading widget…</div> : null}
      {query.isError ? (
        <div className="bergag-error" role="alert">
          {errorMessage(query.error)}
        </div>
      ) : null}
      {query.isSuccess ? (
        <WidgetBody data={query.data} widget={widget} onOpenArticle={onOpenArticle} />
      ) : null}
    </section>
  );
}

function WidgetBody({
  data,
  widget,
  onOpenArticle,
}: {
  data: WidgetData;
  widget: DashboardWidget;
  onOpenArticle: (id: string) => void;
}) {
  if (data === null) {
    return <AnalyticsEmpty title="Unsupported widget" text="This widget type is not available." />;
  }

  switch (widget.kind) {
    case "unread_by_feed":
    case "top_labels":
      return (
        <CountRows
          items={data as NamedCount[]}
          emptyText={
            widget.kind === "unread_by_feed"
              ? "No unread articles are waiting in any feed."
              : "No labels have been assigned to articles yet."
          }
        />
      );
    case "recent_articles":
    case "reading_queue":
      return (
        <ArticleRows
          items={data as ArticleListItem[]}
          onOpenArticle={onOpenArticle}
          emptyText={
            widget.kind === "reading_queue"
              ? "Your reading queue is empty. Add articles from the reader."
              : "No articles have been discovered in this profile yet."
          }
        />
      );
    case "reading_activity": {
      const stats = data as ReadingStats;
      if (stats.activity.every((day) => day.opened === 0 && day.read === 0)) {
        return (
          <AnalyticsEmpty
            title="No reading events yet"
            text="Open or finish an article and its events will appear here."
          />
        );
      }
      return (
        <div className="bergag-widget-stats">
          <StatCard label="Opened" value={stats.total_opened} />
          <StatCard label="Read" value={stats.total_read} />
          <ActivityChart activity={stats.activity.slice(-30)} />
        </div>
      );
    }
    case "feed_health": {
      const feeds = data as FeedHealth[];
      if (feeds.length === 0) {
        return (
          <AnalyticsEmpty
            title="No feeds yet"
            text="Subscribe to a feed to see its local health information."
          />
        );
      }
      return (
        <div className="bergag-count-rows">
          {feeds.map((feed) => (
            <div className="bergag-count-row" key={feed.id}>
              <div className="bergag-count-row__name">
                {feed.title}
                <span className={`bergag-badge bergag-badge--${feed.health}`}>
                  {healthLabel(feed.health)}
                </span>
              </div>
              <span className="bergag-badge">{feed.unread} unread</span>
            </div>
          ))}
        </div>
      );
    }
    case "storage": {
      const storage = data as StorageStats;
      return (
        <div className="bergag-widget-stats">
          <StatCard
            label="Database"
            value={`${(storage.database_bytes / 1024 / 1024).toFixed(1)} MB`}
          />
          <StatCard label="Cached images" value={storage.cached_images} />
          <StatCard label="Extracted articles" value={storage.extracted_articles} />
        </div>
      );
    }
    case "briefing": {
      const briefing = data as Briefing;
      return (
        <div className="bergag-widget-stats">
          <StatCard label="New articles" value={briefing.new_articles} />
          <StatCard label="Read events" value={briefing.read_events} />
          <CountRows items={briefing.top_feeds} emptyText="No feeds contributed unread articles." />
          {briefing.recent_highlights.length > 0 ? (
            <div className="bergag-widget-highlights">
              {briefing.recent_highlights.map((highlight) => (
                <button
                  type="button"
                  key={`${highlight.item_id}:${highlight.created_at}`}
                  className="bergag-related-item"
                  onClick={() => onOpenArticle(highlight.item_id)}
                >
                  <span className="bergag-related-item__title">{highlight.article_title}</span>
                  <span className="bergag-stat__detail">{highlight.text}</span>
                </button>
              ))}
            </div>
          ) : null}
        </div>
      );
    }
    default:
      return null;
  }
}

function ArticleRows({
  items,
  onOpenArticle,
  emptyText,
}: {
  items: ArticleListItem[];
  onOpenArticle: (id: string) => void;
  emptyText: string;
}) {
  if (items.length === 0) {
    return <AnalyticsEmpty title="Nothing to show" text={emptyText} />;
  }
  return (
    <div className="bergag-widget-articles">
      {items.map((item) => (
        <button
          type="button"
          key={item.id}
          className="bergag-related-item"
          onClick={() => onOpenArticle(item.id)}
        >
          <span className="bergag-related-item__title">{item.title}</span>
          <span className="bergag-stat__detail">
            {item.feed_title} · {formatArticleDate(item.discovered_at)}
          </span>
        </button>
      ))}
    </div>
  );
}
