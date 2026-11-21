import type { ActivityDay, ReadingStats } from "../domain/models";

export const READING_RANGES = [7, 30, 90] as const;

export type ReadingRange = (typeof READING_RANGES)[number];

export function formatCount(value: number): string {
  return new Intl.NumberFormat().format(value);
}

export function formatDayLabel(date: string): string {
  const [year, month, day] = date.split("-");
  if (!year || !month || !day) return date;
  return `${Number(month)}/${Number(day)}`;
}

export function activityPeak(activity: ActivityDay[]): number {
  return activity.reduce((peak, day) => Math.max(peak, day.opened, day.read, day.unread), 0);
}

export function activityPercent(value: number, peak: number): number {
  if (peak <= 0 || value <= 0) return 0;
  return Math.max(0, Math.min(100, Math.round((value / peak) * 100)));
}

export function hasReadingActivity(stats: ReadingStats | undefined): boolean {
  if (!stats) return false;
  return stats.total_opened > 0 || stats.total_read > 0 || stats.highlights > 0 || stats.notes > 0;
}

export function countLabel(count: number, singular: string, plural = `${singular}s`): string {
  return `${formatCount(count)} ${count === 1 ? singular : plural}`;
}
