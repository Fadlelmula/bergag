export type AiProviderKind = "openai" | "anthropic" | "gemini" | "ollama" | "openai_compatible";

export interface AiProvider {
  id: string;
  name: string;
  kind: AiProviderKind;
  endpoint: string;
  model: string;
  enabled: boolean;
  has_key: boolean;
  requires_key: boolean;
  created_at: string;
  updated_at: string;
}

export interface AiFeatureSettings {
  summarize: boolean;
  suggestLabels: boolean;
  ask: boolean;
}

export interface AiPrivacySettings {
  allowArticleText: boolean;
  allowHighlights: boolean;
  allowNotes: boolean;
  allowLabels: boolean;
  localProvidersEnabled: boolean;
  requireConfirmation: boolean;
}

export interface AiSettings {
  enabled: boolean;
  providerId: string | null;
  features: AiFeatureSettings;
  privacy: AiPrivacySettings;
}

export interface AiUsageEvent {
  id: string;
  provider_id: string | null;
  provider_name: string;
  provider_kind: AiProviderKind;
  feature: string;
  success: boolean;
  input_chars: number;
  output_chars: number;
  prompt_tokens: number;
  completion_tokens: number;
  latency_ms: number;
  error: string | null;
  created_at: string;
}

export interface AiUsageStats {
  total_requests: number;
  successful_requests: number;
  failed_requests: number;
  total_input_chars: number;
  total_output_chars: number;
  prompt_tokens: number;
  completion_tokens: number;
  last_used_at: string | null;
  by_feature: NamedAiCount[];
  by_provider: NamedAiCount[];
}

export interface NamedAiCount {
  id: string;
  name: string;
  count: number;
  kind: string | null;
}

export interface AiCompletion {
  feature: string;
  text: string;
  provider_id: string;
  provider_name: string;
  model: string;
  prompt_tokens: number;
  completion_tokens: number;
  latency_ms: number;
}

export interface AiTestResult {
  ok: boolean;
  message: string;
  latency_ms: number;
}
