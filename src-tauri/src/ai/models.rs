use crate::analytics::models::NamedCountDto;
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AiProviderKind {
    #[serde(rename = "openai")]
    OpenAi,
    #[serde(rename = "anthropic")]
    Anthropic,
    #[serde(rename = "gemini")]
    Gemini,
    #[serde(rename = "ollama")]
    Ollama,
    #[serde(rename = "openai_compatible")]
    OpenAiCompatible,
}

impl AiProviderKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OpenAi => "openai",
            Self::Anthropic => "anthropic",
            Self::Gemini => "gemini",
            Self::Ollama => "ollama",
            Self::OpenAiCompatible => "openai_compatible",
        }
    }

    pub fn parse(raw: &str) -> AppResult<Self> {
        match raw {
            "openai" => Ok(Self::OpenAi),
            "anthropic" => Ok(Self::Anthropic),
            "gemini" => Ok(Self::Gemini),
            "ollama" => Ok(Self::Ollama),
            "openai_compatible" => Ok(Self::OpenAiCompatible),
            _ => Err(AppError::Validation(
                "Unknown AI provider type.".to_string(),
            )),
        }
    }

    pub fn requires_key(self) -> bool {
        !matches!(self, Self::Ollama)
    }

    pub fn is_local(self) -> bool {
        matches!(self, Self::Ollama)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AiFeature {
    #[serde(rename = "connection_test")]
    ConnectionTest,
    #[serde(rename = "summarize")]
    Summarize,
    #[serde(rename = "suggest_labels")]
    SuggestLabels,
    #[serde(rename = "ask")]
    Ask,
}

impl AiFeature {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ConnectionTest => "connection_test",
            Self::Summarize => "summarize",
            Self::SuggestLabels => "suggest_labels",
            Self::Ask => "ask",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiProviderDto {
    pub id: String,
    pub name: String,
    pub kind: AiProviderKind,
    pub endpoint: String,
    pub model: String,
    pub enabled: bool,
    pub has_key: bool,
    pub requires_key: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AiFeatureSettingsDto {
    pub summarize: bool,
    pub suggest_labels: bool,
    pub ask: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AiPrivacySettingsDto {
    pub allow_article_text: bool,
    pub allow_highlights: bool,
    pub allow_notes: bool,
    pub allow_labels: bool,
    pub local_providers_enabled: bool,
    pub require_confirmation: bool,
}

impl Default for AiPrivacySettingsDto {
    fn default() -> Self {
        Self {
            allow_article_text: false,
            allow_highlights: false,
            allow_notes: false,
            allow_labels: false,
            local_providers_enabled: false,
            require_confirmation: true,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AiSettingsDto {
    pub enabled: bool,
    pub provider_id: Option<String>,
    pub features: AiFeatureSettingsDto,
    pub privacy: AiPrivacySettingsDto,
}

impl AiSettingsDto {
    pub fn validated(mut self) -> AppResult<Self> {
        if let Some(provider_id) = self.provider_id.as_deref() {
            if provider_id.is_empty() || provider_id.chars().count() > 128 {
                return Err(AppError::Validation(
                    "The selected AI provider is invalid.".to_string(),
                ));
            }
        }
        self.provider_id = self
            .provider_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        Ok(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiUsageEventDto {
    pub id: String,
    pub provider_id: Option<String>,
    pub provider_name: String,
    pub provider_kind: AiProviderKind,
    pub feature: String,
    pub success: bool,
    pub input_chars: i64,
    pub output_chars: i64,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub latency_ms: i64,
    pub error: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiUsageStatsDto {
    pub total_requests: i64,
    pub successful_requests: i64,
    pub failed_requests: i64,
    pub total_input_chars: i64,
    pub total_output_chars: i64,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub last_used_at: Option<String>,
    pub by_feature: Vec<NamedCountDto>,
    pub by_provider: Vec<NamedCountDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiCompletionDto {
    pub feature: String,
    pub text: String,
    pub provider_id: String,
    pub provider_name: String,
    pub model: String,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub latency_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AiTestResultDto {
    pub ok: bool,
    pub message: String,
    pub latency_ms: i64,
}

#[derive(Debug, Clone)]
pub struct AiRecordUsage {
    pub provider_id: Option<String>,
    pub provider_name: String,
    pub provider_kind: AiProviderKind,
    pub feature: AiFeature,
    pub success: bool,
    pub input_chars: i64,
    pub output_chars: i64,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub latency_ms: i64,
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_kinds_round_trip() {
        for kind in [
            AiProviderKind::OpenAi,
            AiProviderKind::Anthropic,
            AiProviderKind::Gemini,
            AiProviderKind::Ollama,
            AiProviderKind::OpenAiCompatible,
        ] {
            assert_eq!(AiProviderKind::parse(kind.as_str()).unwrap(), kind);
        }
        assert!(AiProviderKind::parse("unknown").is_err());
    }

    #[test]
    fn ollama_is_the_only_local_provider() {
        assert!(AiProviderKind::Ollama.is_local());
        assert!(!AiProviderKind::Ollama.requires_key());
        assert!(AiProviderKind::OpenAi.requires_key());
    }
}
