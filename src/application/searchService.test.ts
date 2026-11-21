import { describe, expect, it } from "vitest";
import {
  formatStoryBucketDate,
  SEARCH_OPERATOR_HELP,
  SEARCH_SCOPES,
  searchScopeLabel,
} from "./searchService";

describe("search service", () => {
  it("exposes the supported scopes in a stable order", () => {
    expect(SEARCH_SCOPES.map((scope) => scope.id)).toEqual([
      "all",
      "unread",
      "saved",
      "favorites",
      "archive",
      "queue",
    ]);
    expect(searchScopeLabel("saved")).toBe("Read later");
  });

  it("documents every implemented operator", () => {
    const operators = SEARCH_OPERATOR_HELP.map(([operator]) => operator);
    expect(operators).toContain("feed:name");
    expect(operators).toContain("tag:name");
    expect(operators).toContain("is:state");
    expect(operators).toContain("has:value");
    expect(operators).toContain("-term");
    for (const [, description] of SEARCH_OPERATOR_HELP) {
      expect(description.length).toBeGreaterThan(0);
    }
  });

  it("formats known and undated story buckets", () => {
    expect(formatStoryBucketDate("Undated")).toBe("Undated");
    expect(formatStoryBucketDate("not-a-date")).toBe("not-a-date");
    expect(formatStoryBucketDate("2026-11-20").length).toBeGreaterThan(0);
  });
});
