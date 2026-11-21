import { describe, expect, it } from "vitest";
import { aiReadiness, canRunAiFeature } from "./aiService";
import { DEFAULT_AI } from "../domain/settings";
import type { AiProvider, AiSettings } from "../domain/models";

function provider(overrides: Partial<AiProvider> = {}): AiProvider {
  return {
    id: "provider-1",
    name: "Local",
    kind: "ollama",
    endpoint: "http://127.0.0.1:11434",
    model: "llama3.1",
    enabled: true,
    has_key: true,
    requires_key: false,
    created_at: "",
    updated_at: "",
    ...overrides,
  };
}

function enabledSettings(overrides: Partial<AiSettings> = {}): AiSettings {
  const { features, privacy, ...rest } = overrides;
  return {
    ...DEFAULT_AI,
    enabled: true,
    providerId: "provider-1",
    ...rest,
    features: { ...DEFAULT_AI.features, summarize: true, ...features },
    privacy: { ...DEFAULT_AI.privacy, allowArticleText: true, ...privacy },
  };
}

describe("AI feature readiness", () => {
  it("blocks every feature until AI is explicitly enabled", () => {
    expect(canRunAiFeature(DEFAULT_AI, [], "summarize")).toBe(false);
    expect(aiReadiness(DEFAULT_AI, [], "summarize")).toContain("AI is disabled.");
  });

  it("requires a ready selected provider and article-text consent", () => {
    const settings = enabledSettings();
    expect(canRunAiFeature(settings, [], "summarize")).toBe(false);
    expect(canRunAiFeature(settings, [provider()], "summarize")).toBe(false);
    expect(aiReadiness(settings, [provider()], "summarize")).toContain(
      "Enable local providers in AI settings.",
    );
    const ready = enabledSettings({
      privacy: { ...DEFAULT_AI.privacy, allowArticleText: true, localProvidersEnabled: true },
    });
    expect(canRunAiFeature(ready, [provider()], "summarize")).toBe(true);
  });

  it("requires label-name consent for label suggestions", () => {
    const ready = enabledSettings({
      privacy: {
        ...DEFAULT_AI.privacy,
        allowArticleText: true,
        localProvidersEnabled: true,
      },
      features: { ...DEFAULT_AI.features, suggestLabels: true },
    });
    expect(canRunAiFeature(ready, [provider()], "suggestLabels")).toBe(false);
    expect(aiReadiness(ready, [provider()], "suggestLabels")).toContain(
      "Allow label names in AI settings.",
    );
  });

  it("requires a stored key for remote providers", () => {
    const settings = enabledSettings();
    const remote = provider({ kind: "openai", requires_key: true, has_key: false });
    expect(aiReadiness(settings, [remote], "summarize")).toContain(
      "Store an API key for the selected provider.",
    );
  });
});
