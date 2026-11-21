import { useQuery } from "@tanstack/react-query";
import { errorMessage, formatArticleDate } from "../../application/articleService";
import { countLabel, formatCount } from "../../application/analyticsService";
import { queryKeys } from "../../application/queryKeys";
import { desktop } from "../../infrastructure/tauri/commands";
import { useState } from "react";
import { AnalyticsEmpty, CountRows, StatCard } from "./AnalyticsBits";

interface BriefingsViewProps {
  profileId: string;
  onOpenArticle: (id: string) => void;
}

type Period = "daily" | "weekly";

export function BriefingsView({ profileId, onOpenArticle }: BriefingsViewProps) {
  const [period, setPeriod] = useState<Period>("daily");
  const briefing = useQuery({
    queryKey: queryKeys.briefing(profileId, period),
    queryFn: () => desktop.buildBriefing(period),
    enabled: Boolean(profileId),
  });

  return (
    <div className="bergag-full-view">
      <div className="bergag-full-view__header">
        <div>
          <h2 className="bergag-title">Briefings</h2>
          <p className="bergag-subtitle">
            On-demand summaries built from this profile's local articles and annotations. No
            scheduling, delivery, or cloud service is involved.
          </p>
        </div>
        <div className="bergag-segmented" role="group" aria-label="Briefing period">
          {(["daily", "weekly"] as const).map((option) => (
            <button
              type="button"
              key={option}
              aria-pressed={period === option}
              className={`bergag-segmented__item ${
                period === option ? "bergag-segmented__item--active" : ""
              }`}
              onClick={() => setPeriod(option)}
            >
              {option === "daily" ? "Daily" : "Weekly"}
            </button>
          ))}
        </div>
      </div>
      <div className="bergag-full-view__body">
        <div className="bergag-analytics-panel">
          {briefing.isLoading ? <div className="bergag-loading">Building briefing…</div> : null}
          {briefing.isError ? (
            <div className="bergag-error" role="alert">
              {errorMessage(briefing.error)}
            </div>
          ) : null}
          {briefing.data ? (
            <>
              <div className="bergag-briefing-header">
                <div>
                  <h3 className="bergag-section-title">
                    {period === "daily" ? "Daily briefing" : "Weekly briefing"}
                  </h3>
                  <p className="bergag-subtitle">
                    Window starts {formatArticleDate(briefing.data.since)} · generated{" "}
                    {formatArticleDate(briefing.data.generated_at)}
                  </p>
                </div>
                <span className="bergag-badge bergag-badge--accent">
                  {period === "daily" ? "Last 24 hours" : "Last 7 days"}
                </span>
              </div>

              <div className="bergag-card-grid">
                <StatCard
                  label="New articles"
                  value={formatCount(briefing.data.new_articles)}
                  detail="Discovered in the window"
                />
                <StatCard
                  label="Read events"
                  value={formatCount(briefing.data.read_events)}
                  detail="Local read history"
                />
                <StatCard
                  label="Unread total"
                  value={countLabel(briefing.data.unread_total, "article")}
                  detail="Current profile state"
                />
                <StatCard
                  label="Highlights saved"
                  value={formatCount(briefing.data.highlights_saved)}
                  detail="Passages kept in this window"
                />
                <StatCard
                  label="Notes added"
                  value={formatCount(briefing.data.notes_added)}
                  detail="Private notes in this window"
                />
              </div>

              <div className="bergag-briefing-columns">
                <section className="bergag-card bergag-card--padded">
                  <h4 className="bergag-title bergag-title--small">Feeds with unread articles</h4>
                  <CountRows
                    items={briefing.data.top_feeds}
                    emptyText="No feeds have unread articles in this profile."
                  />
                </section>
                <section className="bergag-card bergag-card--padded">
                  <h4 className="bergag-title bergag-title--small">Recent highlights</h4>
                  {briefing.data.recent_highlights.length === 0 ? (
                    <div className="bergag-note">No highlights were saved in this window.</div>
                  ) : (
                    <div className="bergag-widget-highlights">
                      {briefing.data.recent_highlights.map((highlight) => (
                        <button
                          type="button"
                          key={`${highlight.item_id}:${highlight.created_at}`}
                          className="bergag-related-item"
                          onClick={() => onOpenArticle(highlight.item_id)}
                        >
                          <span className="bergag-related-item__title">
                            {highlight.article_title}
                          </span>
                          <span className="bergag-stat__detail">{highlight.text}</span>
                        </button>
                      ))}
                    </div>
                  )}
                </section>
              </div>

              {briefing.data.new_articles === 0 &&
              briefing.data.read_events === 0 &&
              briefing.data.highlights_saved === 0 &&
              briefing.data.notes_added === 0 &&
              briefing.data.top_feeds.length === 0 ? (
                <AnalyticsEmpty
                  title="Quiet period"
                  text="No local activity fell inside this briefing window. Switch to weekly for a wider view."
                />
              ) : null}
            </>
          ) : null}
        </div>
      </div>
    </div>
  );
}
