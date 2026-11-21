import { describe, expect, it } from "vitest";
import {
  availableIntegrationPlatforms,
  integrationDeleteMessage,
  integrationKindLabel,
  integrationPreviewIsCurrent,
  integrationPreviewNotices,
  integrationRemoveMessage,
  plannedIntegrationPlatforms,
  validateIntegrationSelection,
} from "./integrationService";
import type {
  IntegrationPlatform,
  IntegrationPreviewResult,
  IntegrationSource,
} from "../domain/models";

const platforms: IntegrationPlatform[] = [
  {
    kind: "youtube",
    label: "YouTube",
    description: "Public channel feed",
    input_label: "Channel ID",
    input_hint: "UC…",
    available: true,
    unavailable_reason: null,
  },
  {
    kind: "bluesky",
    label: "Bluesky",
    description: "Planned",
    input_label: "Handle",
    input_hint: "Planned",
    available: false,
    unavailable_reason: "Requires an authenticated API; planned.",
  },
];

const source: IntegrationSource = {
  id: "source-1",
  kind: "youtube",
  source_key: "UCabcdefghijklmnopqrstuv",
  display_name: "Example channel",
  source_url: "https://www.youtube.com/channel/UCabcdefghijklmnopqrstuv",
  feed_url: "https://www.youtube.com/feeds/videos.xml?channel_id=UCabcdefghijklmnopqrstuv",
  feed_id: "feed-1",
  category: null,
  trust_level: "low",
  status: "active",
  last_error: null,
  created_at: "2026-01-01T00:00:00Z",
  updated_at: "2026-01-01T00:00:00Z",
};

const preview: IntegrationPreviewResult = {
  platform: platforms[0]!,
  resolved_feed_url: source.feed_url,
  preview: {
    title: "Example channel",
    site_url: null,
    feed_type: "atom",
    description: null,
    language: null,
    item_count: 3,
    items: [],
  },
  duplicate_feed: true,
  duplicate_source: true,
  existing_source: source,
};

describe("integrationService", () => {
  it("separates available and planned platforms", () => {
    expect(availableIntegrationPlatforms(platforms).map((item) => item.kind)).toEqual(["youtube"]);
    expect(plannedIntegrationPlatforms(platforms).map((item) => item.kind)).toEqual(["bluesky"]);
    expect(integrationKindLabel(platforms, "youtube")).toBe("YouTube");
    expect(integrationKindLabel(platforms, "reddit")).toBe("reddit");
  });

  it("validates the selected source before any request", () => {
    expect(validateIntegrationSelection("youtube", " UCabcdefghijklmnopqrstuv ", platforms)).toBe(
      null,
    );
    expect(validateIntegrationSelection("youtube", "  ", platforms)).toBe("Enter a channel id.");
    expect(validateIntegrationSelection("bluesky", "user.bsky.social", platforms)).toBe(
      "Requires an authenticated API; planned.",
    );
    expect(validateIntegrationSelection("", "value", platforms)).toBe("Choose a platform.");
  });

  it("reports duplicate source and feed notices from a real preview", () => {
    expect(integrationPreviewNotices(preview)).toEqual([
      "Already connected as Example channel. Adding this again updates that source.",
      "This public feed is already subscribed. Bergag will reuse the existing feed.",
    ]);
    expect(
      integrationPreviewNotices({
        ...preview,
        duplicate_source: false,
        duplicate_feed: false,
        existing_source: null,
      }),
    ).toEqual([]);
  });

  it("explains that removal keeps the feed and articles", () => {
    expect(integrationDeleteMessage(source)).toContain("feed and its saved articles will remain");
    expect(integrationRemoveMessage(source)).toContain("feed and saved articles remain local");
  });

  it("tracks whether a preview matches the current form", () => {
    const state = { kind: "youtube" as const, input: "UCabcdefghijklmnopqrstuv", result: preview };
    expect(integrationPreviewIsCurrent(state, "youtube", " UCabcdefghijklmnopqrstuv ")).toBe(true);
    expect(integrationPreviewIsCurrent(state, "youtube", "changed")).toBe(false);
    expect(integrationPreviewIsCurrent(null, "youtube", "UCabcdefghijklmnopqrstuv")).toBe(false);
  });
});
