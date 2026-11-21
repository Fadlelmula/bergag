import { describe, expect, it } from "vitest";
import {
  DEFAULT_AI,
  DEFAULT_APPEARANCE,
  DEFAULT_OFFLINE,
  DEFAULT_READING,
  normalizeAi,
  normalizeAppearance,
  normalizeOffline,
  normalizeReading,
} from "./settings";

describe("appearance normalization", () => {
  it("uses safe defaults for missing or invalid values", () => {
    expect(normalizeAppearance(undefined)).toEqual(DEFAULT_APPEARANCE);
    expect(normalizeAppearance({ theme: "neon", accent: "red", density: "huge" })).toEqual(
      DEFAULT_APPEARANCE,
    );
  });

  it("accepts valid persisted values", () => {
    expect(
      normalizeAppearance({
        theme: "light",
        accent: "#4FA77B",
        density: "compact",
        reduceMotion: true,
      }),
    ).toEqual({
      theme: "light",
      accent: "#4fa77b",
      density: "compact",
      reduceMotion: true,
    });
  });
});

describe("reader setting normalization", () => {
  it("keeps automatic read marking off unless explicitly enabled", () => {
    expect(normalizeReading(undefined)).toEqual(DEFAULT_READING);
    expect(normalizeReading({ autoMarkRead: "yes" })).toEqual(DEFAULT_READING);
    expect(normalizeReading({ autoMarkRead: true }).autoMarkRead).toBe(true);
  });

  it("bounds typography and validates presentation", () => {
    expect(normalizeReading({ fontSize: 100, lineHeight: 0.5, presentation: "poster" })).toEqual({
      autoMarkRead: false,
      fontSize: 24,
      lineHeight: 1.3,
      presentation: "list",
    });
    expect(normalizeReading({ fontSize: 16, lineHeight: 1.8, presentation: "magazine" })).toEqual({
      autoMarkRead: false,
      fontSize: 16,
      lineHeight: 1.8,
      presentation: "magazine",
    });
  });

  it("defaults image caching on for offline reading", () => {
    expect(normalizeOffline(null)).toEqual(DEFAULT_OFFLINE);
    expect(normalizeOffline({ cacheImages: false }).cacheImages).toBe(false);
  });
});

describe("AI setting normalization", () => {
  it("keeps every AI opt-in off by default", () => {
    expect(normalizeAi(undefined)).toEqual(DEFAULT_AI);
    expect(normalizeAi({ enabled: "yes", features: { ask: 1 } })).toEqual(DEFAULT_AI);
  });

  it("accepts explicit opt-ins and a selected provider", () => {
    expect(
      normalizeAi({
        enabled: true,
        providerId: " provider-1 ",
        features: { summarize: true, suggestLabels: false, ask: true },
        privacy: {
          allowArticleText: true,
          allowHighlights: true,
          localProvidersEnabled: true,
          requireConfirmation: false,
        },
      }),
    ).toEqual({
      enabled: true,
      providerId: "provider-1",
      features: { summarize: true, suggestLabels: false, ask: true },
      privacy: {
        allowArticleText: true,
        allowHighlights: true,
        allowNotes: false,
        allowLabels: false,
        localProvidersEnabled: true,
        requireConfirmation: false,
      },
    });
  });
});
