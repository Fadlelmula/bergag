import type {
  AiFeatureSettings,
  AiPrivacySettings,
  AiSettings,
  AppearanceSettings,
  ArticlePresentation,
  OfflineSettings,
  ReadingSettings,
} from "./models";

export const SETTINGS_KEYS = {
  appearance: "appearance",
  general: "general",
  reading: "reading",
  offline: "offline",
  ai: "ai.settings",
} as const;

export const ACCENT_PRESETS = [
  { name: "Spider lily", value: "#e34257" },
  { name: "Ember", value: "#d97745" },
  { name: "Jade", value: "#4fa77b" },
  { name: "Azure", value: "#5b8bd9" },
  { name: "Amethyst", value: "#9b72cf" },
] as const;

export const DEFAULT_APPEARANCE: AppearanceSettings = {
  theme: "dark",
  accent: ACCENT_PRESETS[0].value,
  density: "comfortable",
  reduceMotion: false,
};

export const DEFAULT_READING: ReadingSettings = {
  autoMarkRead: false,
  fontSize: 18,
  lineHeight: 1.65,
  presentation: "list",
};

export const DEFAULT_OFFLINE: OfflineSettings = {
  cacheImages: true,
};

export const DEFAULT_AI_FEATURES: AiFeatureSettings = {
  summarize: false,
  suggestLabels: false,
  ask: false,
};

export const DEFAULT_AI_PRIVACY: AiPrivacySettings = {
  allowArticleText: false,
  allowHighlights: false,
  allowNotes: false,
  allowLabels: false,
  localProvidersEnabled: false,
  requireConfirmation: true,
};

export const DEFAULT_AI: AiSettings = {
  enabled: false,
  providerId: null,
  features: DEFAULT_AI_FEATURES,
  privacy: DEFAULT_AI_PRIVACY,
};

export const ARTICLE_PRESENTATIONS: readonly ArticlePresentation[] = [
  "list",
  "magazine",
  "minimal",
];

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isHexColor(value: unknown): value is string {
  return typeof value === "string" && /^#[0-9a-fA-F]{6}$/.test(value);
}

export function normalizeAppearance(value: unknown): AppearanceSettings {
  if (!isRecord(value)) return { ...DEFAULT_APPEARANCE };
  return {
    theme:
      value.theme === "dark" || value.theme === "light" || value.theme === "system"
        ? value.theme
        : DEFAULT_APPEARANCE.theme,
    accent: isHexColor(value.accent) ? value.accent.toLowerCase() : DEFAULT_APPEARANCE.accent,
    density:
      value.density === "compact" || value.density === "comfortable" || value.density === "spacious"
        ? value.density
        : DEFAULT_APPEARANCE.density,
    reduceMotion:
      typeof value.reduceMotion === "boolean"
        ? value.reduceMotion
        : DEFAULT_APPEARANCE.reduceMotion,
  };
}

function boundedNumber(value: unknown, minimum: number, maximum: number, fallback: number): number {
  if (typeof value !== "number" || !Number.isFinite(value)) return fallback;
  return Math.min(maximum, Math.max(minimum, value));
}

export function normalizeReading(value: unknown): ReadingSettings {
  if (!isRecord(value)) return { ...DEFAULT_READING };
  return {
    autoMarkRead:
      typeof value.autoMarkRead === "boolean" ? value.autoMarkRead : DEFAULT_READING.autoMarkRead,
    fontSize: boundedNumber(value.fontSize, 14, 24, DEFAULT_READING.fontSize),
    lineHeight: boundedNumber(value.lineHeight, 1.3, 2, DEFAULT_READING.lineHeight),
    presentation: ARTICLE_PRESENTATIONS.includes(value.presentation as ArticlePresentation)
      ? (value.presentation as ArticlePresentation)
      : DEFAULT_READING.presentation,
  };
}

export function normalizeOffline(value: unknown): OfflineSettings {
  if (!isRecord(value)) return { ...DEFAULT_OFFLINE };
  return {
    cacheImages:
      typeof value.cacheImages === "boolean" ? value.cacheImages : DEFAULT_OFFLINE.cacheImages,
  };
}

function booleanSetting(value: unknown, fallback: boolean): boolean {
  return typeof value === "boolean" ? value : fallback;
}

export function normalizeAi(value: unknown): AiSettings {
  if (!isRecord(value)) {
    return {
      ...DEFAULT_AI,
      features: { ...DEFAULT_AI_FEATURES },
      privacy: { ...DEFAULT_AI_PRIVACY },
    };
  }
  const features = isRecord(value.features) ? value.features : {};
  const privacy = isRecord(value.privacy) ? value.privacy : {};
  const providerId =
    typeof value.providerId === "string" && value.providerId.trim().length > 0
      ? value.providerId.trim()
      : null;
  return {
    enabled: booleanSetting(value.enabled, DEFAULT_AI.enabled),
    providerId,
    features: {
      summarize: booleanSetting(features.summarize, DEFAULT_AI_FEATURES.summarize),
      suggestLabels: booleanSetting(features.suggestLabels, DEFAULT_AI_FEATURES.suggestLabels),
      ask: booleanSetting(features.ask, DEFAULT_AI_FEATURES.ask),
    },
    privacy: {
      allowArticleText: booleanSetting(
        privacy.allowArticleText,
        DEFAULT_AI_PRIVACY.allowArticleText,
      ),
      allowHighlights: booleanSetting(privacy.allowHighlights, DEFAULT_AI_PRIVACY.allowHighlights),
      allowNotes: booleanSetting(privacy.allowNotes, DEFAULT_AI_PRIVACY.allowNotes),
      allowLabels: booleanSetting(privacy.allowLabels, DEFAULT_AI_PRIVACY.allowLabels),
      localProvidersEnabled: booleanSetting(
        privacy.localProvidersEnabled,
        DEFAULT_AI_PRIVACY.localProvidersEnabled,
      ),
      requireConfirmation: booleanSetting(
        privacy.requireConfirmation,
        DEFAULT_AI_PRIVACY.requireConfirmation,
      ),
    },
  };
}
