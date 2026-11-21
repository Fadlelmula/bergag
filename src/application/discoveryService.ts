import type {
  DiscoverySource,
  LabelKind,
  Rule,
  RuleAction,
  RuleCondition,
  RuleDefinition,
  RuleMatchMode,
  TrustLevel,
} from "../domain/models";

export const TRUST_LEVELS: ReadonlyArray<{ value: TrustLevel; label: string; rank: number }> = [
  { value: "untrusted", label: "Untrusted", rank: 0 },
  { value: "low", label: "Low", rank: 1 },
  { value: "medium", label: "Medium", rank: 2 },
  { value: "high", label: "High", rank: 3 },
];

export function trustLabel(level: TrustLevel): string {
  return TRUST_LEVELS.find((entry) => entry.value === level)?.label ?? "Untrusted";
}

export function labelKindLabel(kind: LabelKind): string {
  return kind === "tag" ? "Tag" : "Topic";
}

export function filterDiscoverySources(
  sources: readonly DiscoverySource[],
  query: string,
  category: string | null,
): DiscoverySource[] {
  const needle = query.trim().toLowerCase();
  return sources.filter((source) => {
    if (category && source.category !== category) return false;
    if (!needle) return true;
    return [source.title, source.feed_url, source.site_url ?? "", source.category ?? ""]
      .join(" ")
      .toLowerCase()
      .includes(needle);
  });
}

export function discoveryCategories(sources: readonly DiscoverySource[]): string[] {
  const categories = new Set<string>();
  for (const source of sources) {
    if (source.category) categories.add(source.category);
  }
  return [...categories].sort((a, b) => a.localeCompare(b));
}

export function defaultRuleDefinition(): RuleDefinition {
  return {
    matchMode: "all",
    conditions: [{ field: "titleContains", value: "" }],
    actions: [{ action: "markRead", value: true }],
  };
}

const CONDITION_FIELDS = new Set<RuleCondition["field"]>([
  "titleContains",
  "summaryContains",
  "contentContains",
  "feedIs",
  "feedTitleContains",
  "categoryIs",
  "trustAtLeast",
  "hasLabel",
  "isRead",
  "isSaved",
  "isFavorite",
  "isArchived",
  "isQueued",
]);

const ACTION_KINDS = new Set<RuleAction["action"]>([
  "markRead",
  "setSaved",
  "setFavorite",
  "setArchived",
  "setQueued",
  "addLabel",
  "removeLabel",
]);

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

export function parseRuleDefinition(raw: string): RuleDefinition | null {
  let parsed: unknown;
  try {
    parsed = JSON.parse(raw);
  } catch {
    return null;
  }
  if (!isRecord(parsed)) return null;
  const matchMode = parsed.matchMode;
  if (matchMode !== "all" && matchMode !== "any") return null;
  if (!Array.isArray(parsed.conditions) || !Array.isArray(parsed.actions)) return null;
  if (parsed.conditions.length === 0 || parsed.actions.length === 0) return null;
  for (const condition of parsed.conditions) {
    if (!isRecord(condition) || typeof condition.field !== "string") return null;
    if (!CONDITION_FIELDS.has(condition.field as RuleCondition["field"])) return null;
  }
  for (const action of parsed.actions) {
    if (!isRecord(action) || typeof action.action !== "string") return null;
    if (!ACTION_KINDS.has(action.action as RuleAction["action"])) return null;
  }
  return parsed as unknown as RuleDefinition;
}

export function ruleSummary(rule: Rule): string {
  const definition = parseRuleDefinition(rule.rule_json);
  if (!definition) return "Rule definition needs repair";
  const conditionCount = definition.conditions.length;
  const actionCount = definition.actions.length;
  return `${definition.matchMode === "all" ? "All" : "Any"} of ${conditionCount} condition${
    conditionCount === 1 ? "" : "s"
  } → ${actionCount} action${actionCount === 1 ? "" : "s"}`;
}

export function conditionLabel(condition: RuleCondition): string {
  switch (condition.field) {
    case "titleContains":
      return `Title contains “${condition.value}”`;
    case "summaryContains":
      return `Summary contains “${condition.value}”`;
    case "contentContains":
      return `Content contains “${condition.value}”`;
    case "feedIs":
      return "From selected feed";
    case "feedTitleContains":
      return `Feed title contains “${condition.value}”`;
    case "categoryIs":
      return `Category is “${condition.value}”`;
    case "trustAtLeast":
      return `Trust at least ${trustLabel(condition.value)}`;
    case "hasLabel":
      return "Has selected label";
    case "isRead":
      return condition.value ? "Is read" : "Is unread";
    case "isSaved":
      return condition.value ? "Is saved" : "Is not saved";
    case "isFavorite":
      return condition.value ? "Is favorite" : "Is not favorite";
    case "isArchived":
      return condition.value ? "Is archived" : "Is not archived";
    case "isQueued":
      return condition.value ? "Is queued" : "Is not queued";
  }
}

export function actionLabel(action: RuleAction): string {
  switch (action.action) {
    case "markRead":
      return action.value ? "Mark read" : "Mark unread";
    case "setSaved":
      return action.value ? "Save" : "Remove from read later";
    case "setFavorite":
      return action.value ? "Favorite" : "Remove favorite";
    case "setArchived":
      return action.value ? "Archive" : "Unarchive";
    case "setQueued":
      return action.value ? "Add to queue" : "Remove from queue";
    case "addLabel":
      return "Add selected label";
    case "removeLabel":
      return "Remove selected label";
  }
}

export function newRuleMatchMode(value: string): RuleMatchMode {
  return value === "any" ? "any" : "all";
}
