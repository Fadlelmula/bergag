import { describe, expect, it } from "vitest";
import {
  conditionLabel,
  discoveryCategories,
  filterDiscoverySources,
  parseRuleDefinition,
  ruleSummary,
  trustLabel,
} from "./discoveryService";
import type { DiscoverySource, Rule } from "../domain/models";

function source(overrides: Partial<DiscoverySource> = {}): DiscoverySource {
  return {
    id: "source-1",
    feed_url: "https://example.com/feed.xml",
    site_url: "https://example.com",
    title: "Example",
    category: "News",
    description: null,
    trust_level: "medium",
    subscribed_feed_id: null,
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
    ...overrides,
  };
}

function rule(ruleJson: string): Rule {
  return {
    id: "rule-1",
    name: "Security",
    description: null,
    rule_json: ruleJson,
    enabled: true,
    is_smart_feed: false,
    last_run_at: null,
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
  };
}

describe("discoveryService", () => {
  it("filters sources by text and category", () => {
    const sources = [
      source(),
      source({ id: "source-2", title: "Rust weekly", category: "Security" }),
      source({ id: "source-3", title: "Local paper", category: null }),
    ];
    expect(filterDiscoverySources(sources, "rust", null).map((item) => item.id)).toEqual([
      "source-2",
    ]);
    expect(filterDiscoverySources(sources, "", "News").map((item) => item.id)).toEqual([
      "source-1",
    ]);
    expect(filterDiscoverySources(sources, "local", "Security")).toEqual([]);
    expect(discoveryCategories(sources)).toEqual(["News", "Security"]);
  });

  it("labels trust levels", () => {
    expect(trustLabel("untrusted")).toBe("Untrusted");
    expect(trustLabel("high")).toBe("High");
  });

  it("parses valid rule definitions and rejects malformed ones", () => {
    const valid = parseRuleDefinition(
      '{"matchMode":"all","conditions":[{"field":"titleContains","value":"rust"}],"actions":[{"action":"markRead","value":true}]}',
    );
    expect(valid?.conditions).toHaveLength(1);
    expect(parseRuleDefinition("not json")).toBeNull();
    expect(
      parseRuleDefinition(
        '{"matchMode":"all","conditions":[{"field":"unknown"}],"actions":[{"action":"markRead","value":true}]}',
      ),
    ).toBeNull();
    expect(
      parseRuleDefinition(
        '{"matchMode":"all","conditions":[],"actions":[{"action":"markRead","value":true}]}',
      ),
    ).toBeNull();
  });

  it("summarizes rules and formats conditions", () => {
    const summary = ruleSummary(
      rule(
        '{"matchMode":"any","conditions":[{"field":"isRead","value":false},{"field":"hasLabel","labelId":"label-1"}],"actions":[{"action":"setSaved","value":true}]}',
      ),
    );
    expect(summary).toBe("Any of 2 conditions → 1 action");
    expect(conditionLabel({ field: "isRead", value: false })).toBe("Is unread");
    expect(conditionLabel({ field: "trustAtLeast", value: "high" })).toBe("Trust at least High");
    expect(ruleSummary(rule("bad json"))).toBe("Rule definition needs repair");
  });
});
