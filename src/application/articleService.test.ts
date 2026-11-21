import { describe, expect, it } from "vitest";
import type { ArticleState } from "../domain/models";
import {
  ARTICLE_MODES,
  articleEmptyMessage,
  articleModeFromNav,
  articleModeLabel,
  articleStateBadges,
  formatBytes,
  isArticleView,
} from "./articleService";

const baseState: ArticleState = {
  is_read: false,
  is_saved: false,
  is_favorite: false,
  is_archived: false,
  is_queued: false,
  queue_position: null,
  read_count: 0,
  last_read_at: null,
};

describe("article presentation helpers", () => {
  it("maps Milestone 3 navigation to real article list modes", () => {
    expect(articleModeFromNav("readLater")).toBe("saved");
    expect(articleModeFromNav("queue")).toBe("queue");
    expect(articleModeFromNav("home")).toBe("unread");
    expect(isArticleView("favorites")).toBe(true);
    expect(isArticleView("allFeeds")).toBe(false);
  });

  it("labels every supported list mode", () => {
    for (const mode of ARTICLE_MODES) {
      expect(articleModeLabel(mode).length).toBeGreaterThan(0);
      expect(articleEmptyMessage(mode).length).toBeGreaterThan(0);
    }
  });

  it("derives state badges from real stored state", () => {
    expect(articleStateBadges(baseState)).toEqual(["Unread"]);
    expect(
      articleStateBadges({
        ...baseState,
        is_read: true,
        is_saved: true,
        is_favorite: true,
        is_queued: true,
        queue_position: 3,
      }),
    ).toEqual(["Saved", "Favorite", "Queue 3"]);
  });

  it("formats cache sizes", () => {
    expect(formatBytes(512)).toBe("512 B");
    expect(formatBytes(2048)).toBe("2.0 KB");
    expect(formatBytes(2 * 1024 * 1024)).toBe("2.0 MB");
  });
});
