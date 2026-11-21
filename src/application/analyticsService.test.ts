import { describe, expect, it } from "vitest";
import type { ActivityDay, ReadingStats } from "../domain/models";
import {
  activityPercent,
  activityPeak,
  countLabel,
  formatCount,
  formatDayLabel,
  hasReadingActivity,
  READING_RANGES,
} from "./analyticsService";

function day(date: string, opened: number, read: number, unread = 0): ActivityDay {
  return { date, opened, read, unread };
}

function stats(overrides: Partial<ReadingStats> = {}): ReadingStats {
  return {
    days: 7,
    activity: [day("2026-11-20", 1, 2)],
    total_opened: 1,
    total_read: 2,
    highlights: 0,
    notes: 0,
    streak_days: 1,
    last_read_at: null,
    ...overrides,
  };
}

describe("analyticsService", () => {
  it("offers only supported reading ranges", () => {
    expect(READING_RANGES).toEqual([7, 30, 90]);
  });

  it("formats counts and day labels without locale-dependent surprises", () => {
    expect(formatCount(1234)).toBe(new Intl.NumberFormat().format(1234));
    expect(formatDayLabel("2026-11-20")).toBe("11/20");
    expect(formatDayLabel("invalid")).toBe("invalid");
    expect(countLabel(1, "article")).toBe("1 article");
    expect(countLabel(2, "article")).toBe("2 articles");
  });

  it("scales chart values against the observed peak", () => {
    const activity = [day("2026-11-19", 2, 4), day("2026-11-20", 1, 2)];
    expect(activityPeak(activity)).toBe(4);
    expect(activityPercent(4, 4)).toBe(100);
    expect(activityPercent(2, 4)).toBe(50);
    expect(activityPercent(1, 4)).toBe(25);
    expect(activityPercent(3, 0)).toBe(0);
    expect(activityPercent(-1, 4)).toBe(0);
  });

  it("detects whether reading history contains any local activity", () => {
    expect(hasReadingActivity(stats())).toBe(true);
    expect(
      hasReadingActivity(
        stats({ total_opened: 0, total_read: 0, highlights: 0, notes: 0, activity: [] }),
      ),
    ).toBe(false);
    expect(hasReadingActivity(undefined)).toBe(false);
  });
});
