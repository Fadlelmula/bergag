import { describe, expect, it } from "vitest";
import {
  feedTypeLabel,
  formatRelativeTime,
  formatSchedule,
  healthLabel,
  prepareFeedUrl,
  refreshIntervalLabel,
} from "./feedService";

describe("feed presentation helpers", () => {
  it("prepares URLs by trimming surrounding whitespace", () => {
    expect(prepareFeedUrl("  https://example.com/feed  ")).toBe("https://example.com/feed");
  });

  it("formats relative time with a stable reference", () => {
    const reference = Date.parse("2026-11-20T12:00:00Z");
    expect(formatRelativeTime(null, reference)).toBe("never");
    expect(formatRelativeTime("not-a-date", reference)).toBe("unknown");
    expect(formatRelativeTime("2026-11-20T11:59:30Z", reference)).toBe("just now");
    expect(formatRelativeTime("2026-11-20T11:00:00Z", reference)).toBe("1h ago");
    expect(formatRelativeTime("2026-11-18T12:00:00Z", reference)).toBe("2d ago");
  });

  it("formats future schedules and overdue feeds", () => {
    const reference = Date.parse("2026-11-20T12:00:00Z");
    expect(formatSchedule(null, reference)).toBe("not scheduled");
    expect(formatSchedule("not-a-date", reference)).toBe("unknown");
    expect(formatSchedule("2026-11-20T11:59:00Z", reference)).toBe("overdue");
    expect(formatSchedule("2026-11-20T12:00:30Z", reference)).toBe("soon");
    expect(formatSchedule("2026-11-20T12:45:00Z", reference)).toBe("in 45m");
    expect(formatSchedule("2026-11-21T12:00:00Z", reference)).toBe("in 1d");
  });

  it("labels health and feed types", () => {
    expect(healthLabel("healthy")).toBe("Healthy");
    expect(healthLabel("degraded")).toBe("Degraded");
    expect(healthLabel("failing")).toBe("Failing");
    expect(feedTypeLabel("rss")).toBe("RSS");
    expect(feedTypeLabel("atom")).toBe("Atom");
    expect(feedTypeLabel("json")).toBe("JSON");
    expect(feedTypeLabel("unknown")).toBe("Unknown");
  });

  it("labels refresh intervals", () => {
    expect(refreshIntervalLabel(15)).toBe("15 min");
    expect(refreshIntervalLabel(60)).toBe("1 hour");
    expect(refreshIntervalLabel(120)).toBe("2 hours");
    expect(refreshIntervalLabel(1440)).toBe("1 day");
  });
});
