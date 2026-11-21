import type { AiFeatureSettings, AiProvider, AiProviderKind, AiSettings } from "../domain/models";

export type AiFeatureId = keyof AiFeatureSettings;

export const AI_FEATURE_LABELS: Record<AiFeatureId, string> = {
  summarize: "Summarize article",
  suggestLabels: "Suggest labels",
  ask: "Ask about article",
};

export const AI_PROVIDER_PRESETS: Record<
  AiProviderKind,
  { label: string; endpoint: string; model: string; requiresKey: boolean }
> = {
  openai: {
    label: "OpenAI",
    endpoint: "https://api.openai.com/v1/chat/completions",
    model: "gpt-4o-mini",
    requiresKey: true,
  },
  anthropic: {
    label: "Anthropic",
    endpoint: "https://api.anthropic.com/v1/messages",
    model: "claude-3-5-haiku-latest",
    requiresKey: true,
  },
  gemini: {
    label: "Google Gemini",
    endpoint: "https://generativelanguage.googleapis.com/v1beta",
    model: "gemini-2.0-flash",
    requiresKey: true,
  },
  ollama: {
    label: "Ollama (local)",
    endpoint: "http://127.0.0.1:11434",
    model: "llama3.1",
    requiresKey: false,
  },
  openai_compatible: {
    label: "OpenAI-compatible endpoint",
    endpoint: "https://api.openai.com/v1/chat/completions",
    model: "gpt-4o-mini",
    requiresKey: true,
  },
};

export const AI_PROVIDER_KINDS = Object.keys(AI_PROVIDER_PRESETS) as AiProviderKind[];

export function aiFeatureEnabled(settings: AiSettings, feature: AiFeatureId): boolean {
  return settings.features[feature];
}

export function selectedAiProvider(
  settings: AiSettings,
  providers: AiProvider[],
): AiProvider | null {
  if (!settings.providerId) return null;
  return providers.find((provider) => provider.id === settings.providerId) ?? null;
}

export function aiReadiness(
  settings: AiSettings,
  providers: AiProvider[],
  feature: AiFeatureId,
): string[] {
  const issues: string[] = [];
  if (!settings.enabled) issues.push("AI is disabled.");
  const provider = selectedAiProvider(settings, providers);
  if (!provider) issues.push("Select a configured provider.");
  if (provider && !provider.enabled) issues.push("The selected provider is disabled.");
  if (provider && provider.requires_key && !provider.has_key) {
    issues.push("Store an API key for the selected provider.");
  }
  if (provider?.kind === "ollama" && !settings.privacy.localProvidersEnabled) {
    issues.push("Enable local providers in AI settings.");
  }
  if (!aiFeatureEnabled(settings, feature)) issues.push("Turn on this feature in AI settings.");
  if (!settings.privacy.allowArticleText) issues.push("Allow article text in AI settings.");
  if (feature === "suggestLabels" && !settings.privacy.allowLabels) {
    issues.push("Allow label names in AI settings.");
  }
  return issues;
}

export function canRunAiFeature(
  settings: AiSettings,
  providers: AiProvider[],
  feature: AiFeatureId,
): boolean {
  return aiReadiness(settings, providers, feature).length === 0;
}
