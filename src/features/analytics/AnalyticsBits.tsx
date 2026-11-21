import {
  activityPeak,
  activityPercent,
  formatCount,
  formatDayLabel,
} from "../../application/analyticsService";
import { Inbox } from "lucide-react";
import type { ActivityDay, NamedCount } from "../../domain/models";

export function StatCard({
  label,
  value,
  detail,
}: {
  label: string;
  value: string | number;
  detail?: string;
}) {
  return (
    <div className="bergag-card bergag-card--padded">
      <div className="bergag-stat__label">{label}</div>
      <div className="bergag-stat__value">{value}</div>
      {detail ? <div className="bergag-stat__detail">{detail}</div> : null}
    </div>
  );
}

export function AnalyticsEmpty({ title, text }: { title: string; text: string }) {
  return (
    <div className="bergag-empty-state">
      <Inbox size={24} className="bergag-empty-state__mark" aria-hidden="true" />
      <p className="bergag-empty-state__title">{title}</p>
      <p className="bergag-empty-state__text">{text}</p>
    </div>
  );
}

export function ActivityChart({ activity }: { activity: ActivityDay[] }) {
  const peak = activityPeak(activity);
  return (
    <div
      className="bergag-activity-chart"
      role="img"
      aria-label={`Reading activity over ${activity.length} days`}
    >
      {activity.map((day) => (
        <div
          className="bergag-activity-column"
          key={day.date}
          title={`${day.date}: ${day.read} read`}
        >
          <div className="bergag-activity-bars">
            <span
              className="bergag-activity-bar bergag-activity-bar--opened"
              style={{ height: `${activityPercent(day.opened, peak)}%` }}
            />
            <span
              className="bergag-activity-bar bergag-activity-bar--read"
              style={{ height: `${activityPercent(day.read, peak)}%` }}
            />
          </div>
          <span className="bergag-activity-day">{formatDayLabel(day.date)}</span>
        </div>
      ))}
    </div>
  );
}

export function CountRows({ items, emptyText }: { items: NamedCount[]; emptyText: string }) {
  if (items.length === 0) {
    return <div className="bergag-note">{emptyText}</div>;
  }
  return (
    <div className="bergag-count-rows">
      {items.map((item) => (
        <div className="bergag-count-row" key={`${item.kind ?? "item"}:${item.id}`}>
          <div className="bergag-count-row__name">
            {item.name}
            {item.kind ? <span className="bergag-badge">{item.kind}</span> : null}
          </div>
          <span className="bergag-badge">{formatCount(item.count)}</span>
        </div>
      ))}
    </div>
  );
}
