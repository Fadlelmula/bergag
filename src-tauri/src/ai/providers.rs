use super::models::{AiProviderDto, AiProviderKind};
use crate::error::{AppError, AppResult};
use crate::feeds::network;
use crate::security::redaction::redact;
use reqwest::redirect::Policy;
use serde_json::{json, Value};
use std::time::Duration;
use url::Url;

const AI_REQUEST_TIMEOUT_SECONDS: u64 = 45;
const MAX_RESPONSE_BYTES: usize = 2 * 1024 * 1024;
const MAX_TEXT_CHARS: usize = 8_000;

#[derive(Debug, Clone)]
pub struct ProviderRequest<'a> {
    pub provider: &'a AiProviderDto,
    pub secret: Option<&'a str>,
    pub system: &'a str,
    pub prompt: &'a str,
    pub max_tokens: usize,
}

#[derive(Debug, Clone)]
pub struct ProviderCompletion {
    pub text: String,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
}

fn has_control(value: &str) -> bool {
    value.chars().any(char::is_control)
}

fn validate_model(model: &str) -> AppResult<()> {
    let trimmed = model.trim();
    if trimmed.is_empty() || trimmed.chars().count() > 128 {
        return Err(AppError::Validation(
            "Model must be between 1 and 128 characters.".to_string(),
        ));
    }
    if has_control(trimmed)
        || !trimmed
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "._:-".contains(character))
    {
        return Err(AppError::Validation(
            "Model contains unsupported characters.".to_string(),
        ));
    }
    Ok(())
}

pub fn validate_provider_config(
    kind: AiProviderKind,
    endpoint: &str,
    model: &str,
    local_providers_enabled: bool,
) -> AppResult<()> {
    let endpoint = endpoint.trim();
    if endpoint.is_empty() || endpoint.chars().count() > 512 || has_control(endpoint) {
        return Err(AppError::Validation(
            "Endpoint must be between 1 and 512 characters.".to_string(),
        ));
    }
    validate_model(model)?;

    if kind.is_local() {
        if !local_providers_enabled {
            return Err(AppError::Validation(
                "Enable local AI providers in Settings → AI before adding Ollama.".to_string(),
            ));
        }
        let url = Url::parse(endpoint).map_err(|_| {
            AppError::Validation("Enter a valid local Ollama endpoint.".to_string())
        })?;
        let host = url.host_str().unwrap_or_default();
        let host = host.trim_start_matches('[').trim_end_matches(']');
        let loopback = host == "127.0.0.1" || host == "::1";
        let port_ok = url
            .port()
            .is_some_and(|port| (1024..=65535).contains(&port));
        let path_ok = url.path().is_empty() || url.path() == "/";
        if url.scheme() != "http"
            || !loopback
            || !port_ok
            || !path_ok
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err(AppError::Validation(
                "Ollama must use a literal loopback http address such as http://127.0.0.1:11434."
                    .to_string(),
            ));
        }
        return Ok(());
    }

    let url = network::parse_public_url(endpoint)?;
    if url.scheme() != "https" {
        return Err(AppError::Validation(
            "Remote AI endpoints must use https.".to_string(),
        ));
    }
    Ok(())
}

fn request_url(provider: &AiProviderDto) -> AppResult<Url> {
    let endpoint = provider.endpoint.trim();
    match provider.kind {
        AiProviderKind::OpenAi | AiProviderKind::OpenAiCompatible | AiProviderKind::Anthropic => {
            Url::parse(endpoint).map_err(|_| {
                AppError::Validation("The AI endpoint is not a valid URL.".to_string())
            })
        }
        AiProviderKind::Gemini => {
            let base = endpoint.trim_end_matches('/');
            Url::parse(&format!("{base}/models/{}:generateContent", provider.model)).map_err(|_| {
                AppError::Validation("The Gemini endpoint is not a valid URL.".to_string())
            })
        }
        AiProviderKind::Ollama => {
            Url::parse(&format!("{}/api/chat", endpoint.trim_end_matches('/')))
                .map_err(|_| AppError::Validation("The Ollama endpoint is not valid.".to_string()))
        }
    }
}

fn build_body(request: &ProviderRequest<'_>) -> AppResult<Value> {
    let model = request.provider.model.as_str();
    let system = request.system;
    let prompt = request.prompt;
    match request.provider.kind {
        AiProviderKind::OpenAi | AiProviderKind::OpenAiCompatible => Ok(json!({
            "model": model,
            "messages": [
                { "role": "system", "content": system },
                { "role": "user", "content": prompt }
            ],
            "temperature": 0.2,
            "max_tokens": request.max_tokens,
        })),
        AiProviderKind::Anthropic => Ok(json!({
            "model": model,
            "system": system,
            "messages": [{ "role": "user", "content": prompt }],
            "temperature": 0.2,
            "max_tokens": request.max_tokens,
        })),
        AiProviderKind::Gemini => Ok(json!({
            "contents": [{ "parts": [{ "text": format!("{system}\n\n{prompt}") }] }],
            "generationConfig": {
                "temperature": 0.2,
                "maxOutputTokens": request.max_tokens,
            },
        })),
        AiProviderKind::Ollama => Ok(json!({
            "model": model,
            "stream": false,
            "system": system,
            "prompt": prompt,
            "options": {
                "temperature": 0.2,
                "num_predict": request.max_tokens,
            },
        })),
    }
}

async fn build_client(url: &Url) -> AppResult<reqwest::Client> {
    let builder = reqwest::Client::builder()
        .user_agent(concat!("Bergag/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(AI_REQUEST_TIMEOUT_SECONDS))
        .connect_timeout(Duration::from_secs(10));
    if url.host_str().unwrap_or_default() == "127.0.0.1"
        || url.host_str().unwrap_or_default() == "[::1]"
        || url.host_str().unwrap_or_default() == "::1"
    {
        builder.redirect(Policy::none()).build().map_err(|_| {
            AppError::Internal("The local HTTP client could not be built.".to_string())
        })
    } else {
        network::client_for(url).await
    }
}

fn apply_auth(
    mut request: reqwest::RequestBuilder,
    kind: AiProviderKind,
    secret: Option<&str>,
) -> reqwest::RequestBuilder {
    let Some(secret) = secret else {
        return request;
    };
    match kind {
        AiProviderKind::OpenAi | AiProviderKind::OpenAiCompatible => {
            request = request.header("authorization", format!("Bearer {secret}"));
        }
        AiProviderKind::Anthropic => {
            request = request.header("x-api-key", secret);
            request = request.header("anthropic-version", "2023-06-01");
        }
        AiProviderKind::Gemini => {
            request = request.header("x-goog-api-key", secret);
        }
        AiProviderKind::Ollama => {}
    }
    request
}

fn json_number(value: Option<&Value>) -> i64 {
    value.and_then(Value::as_i64).unwrap_or(0).max(0)
}

fn parse_response(kind: AiProviderKind, body: &str) -> AppResult<ProviderCompletion> {
    let value: Value = serde_json::from_str(body).map_err(|_| {
        AppError::Internal("The AI provider response could not be read.".to_string())
    })?;
    let text = match kind {
        AiProviderKind::OpenAi | AiProviderKind::OpenAiCompatible => value
            .get("choices")
            .and_then(Value::as_array)
            .and_then(|choices| choices.first())
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(Value::as_str)
            .map(str::to_string),
        AiProviderKind::Anthropic => value
            .get("content")
            .and_then(Value::as_array)
            .and_then(|blocks| {
                blocks
                    .iter()
                    .find_map(|block| block.get("text").and_then(Value::as_str))
            })
            .map(str::to_string),
        AiProviderKind::Gemini => value
            .get("candidates")
            .and_then(Value::as_array)
            .and_then(|candidates| candidates.first())
            .and_then(|candidate| candidate.get("content"))
            .and_then(|content| content.get("parts"))
            .and_then(Value::as_array)
            .and_then(|parts| {
                let joined = parts
                    .iter()
                    .filter_map(|part| part.get("text").and_then(Value::as_str))
                    .collect::<Vec<_>>()
                    .join("\\n");
                (!joined.is_empty()).then_some(joined)
            }),
        AiProviderKind::Ollama => value
            .get("message")
            .and_then(|message| message.get("content"))
            .and_then(Value::as_str)
            .map(str::to_string),
    };

    let text = text.unwrap_or_default();
    let text = text.trim();
    if text.is_empty() {
        return Err(AppError::Internal(
            "The AI provider returned an empty response.".to_string(),
        ));
    }
    let text = text.chars().take(MAX_TEXT_CHARS).collect::<String>();

    let (prompt_tokens, completion_tokens) = match kind {
        AiProviderKind::OpenAi | AiProviderKind::OpenAiCompatible => {
            let usage = value.get("usage");
            (
                json_number(usage.and_then(|entry| entry.get("prompt_tokens"))),
                json_number(usage.and_then(|entry| entry.get("completion_tokens"))),
            )
        }
        AiProviderKind::Anthropic => {
            let usage = value.get("usage");
            (
                json_number(usage.and_then(|entry| entry.get("input_tokens"))),
                json_number(usage.and_then(|entry| entry.get("output_tokens"))),
            )
        }
        AiProviderKind::Gemini => {
            let usage = value.get("usageMetadata");
            (
                json_number(usage.and_then(|entry| entry.get("promptTokenCount"))),
                json_number(usage.and_then(|entry| entry.get("candidatesTokenCount"))),
            )
        }
        AiProviderKind::Ollama => (
            json_number(value.get("prompt_eval_count")),
            json_number(value.get("eval_count")),
        ),
    };

    Ok(ProviderCompletion {
        text,
        prompt_tokens,
        completion_tokens,
    })
}

pub async fn complete(request: ProviderRequest<'_>) -> AppResult<ProviderCompletion> {
    let url = request_url(request.provider)?;
    let body = build_body(&request)?;
    let serialized = serde_json::to_string(&body)?;
    let client = build_client(&url).await?;
    let mut http_request = client
        .post(url)
        .header("content-type", "application/json")
        .body(serialized);
    http_request = apply_auth(http_request, request.provider.kind, request.secret);

    let response = http_request.send().await.map_err(|error| {
        tracing::warn!(error = %redact(&error.to_string()), "AI provider request failed");
        AppError::Internal("The AI provider could not be reached.".to_string())
    })?;
    let status = response.status();
    if !status.is_success() {
        return Err(AppError::Internal(format!(
            "The AI provider returned HTTP {}.",
            status.as_u16()
        )));
    }
    if let Some(length) = response.content_length() {
        if length as usize > MAX_RESPONSE_BYTES {
            return Err(AppError::Validation(
                "The AI provider response is too large.".to_string(),
            ));
        }
    }
    let bytes = response.bytes().await.map_err(|error| {
        tracing::warn!(error = %redact(&error.to_string()), "AI provider response read failed");
        AppError::Internal("The AI provider response could not be read.".to_string())
    })?;
    if bytes.len() > MAX_RESPONSE_BYTES {
        return Err(AppError::Validation(
            "The AI provider response is too large.".to_string(),
        ));
    }
    let body = String::from_utf8_lossy(&bytes);
    parse_response(request.provider.kind, &body)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn provider(kind: AiProviderKind, endpoint: &str, model: &str) -> AiProviderDto {
        AiProviderDto {
            id: "provider-1".to_string(),
            name: "Test".to_string(),
            kind,
            endpoint: endpoint.to_string(),
            model: model.to_string(),
            enabled: true,
            has_key: false,
            requires_key: kind.requires_key(),
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    #[test]
    fn remote_endpoints_must_be_public_https() {
        assert!(validate_provider_config(
            AiProviderKind::OpenAi,
            "https://api.openai.com/v1/chat/completions",
            "gpt-4o-mini",
            false,
        )
        .is_ok());
        assert!(validate_provider_config(
            AiProviderKind::OpenAi,
            "http://127.0.0.1:8080/v1/chat/completions",
            "gpt-4o-mini",
            false,
        )
        .is_err());
        assert!(validate_provider_config(
            AiProviderKind::OpenAi,
            "https://localhost/v1",
            "gpt-4o-mini",
            false,
        )
        .is_err());
    }

    #[test]
    fn ollama_requires_opt_in_and_literal_loopback() {
        assert!(validate_provider_config(
            AiProviderKind::Ollama,
            "http://127.0.0.1:11434",
            "llama3.1",
            false,
        )
        .is_err());
        assert!(validate_provider_config(
            AiProviderKind::Ollama,
            "http://127.0.0.1:11434",
            "llama3.1",
            true,
        )
        .is_ok());
        assert!(validate_provider_config(
            AiProviderKind::Ollama,
            "http://example.com:11434",
            "llama3.1",
            true,
        )
        .is_err());
        assert!(validate_provider_config(
            AiProviderKind::Ollama,
            "http://127.0.0.1:80",
            "llama3.1",
            true,
        )
        .is_err());
    }

    #[test]
    fn gemini_request_url_includes_model() {
        let provider = provider(
            AiProviderKind::Gemini,
            "https://generativelanguage.googleapis.com/v1beta",
            "gemini-2.0-flash",
        );
        assert_eq!(
            request_url(&provider).unwrap().as_str(),
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent"
        );
    }

    #[test]
    fn parses_provider_responses() {
        let openai = r#"{"choices":[{"message":{"content":"Hello"}}],"usage":{"prompt_tokens":8,"completion_tokens":3}}"#;
        let completion = parse_response(AiProviderKind::OpenAi, openai).unwrap();
        assert_eq!(completion.text, "Hello");
        assert_eq!(completion.prompt_tokens, 8);
        assert_eq!(completion.completion_tokens, 3);

        let anthropic = r#"{"content":[{"type":"text","text":"Bonjour"}],"usage":{"input_tokens":5,"output_tokens":2}}"#;
        assert_eq!(
            parse_response(AiProviderKind::Anthropic, anthropic)
                .unwrap()
                .text,
            "Bonjour"
        );

        let gemini = r#"{"candidates":[{"content":{"parts":[{"text":"Hi"}]}}],"usageMetadata":{"promptTokenCount":4,"candidatesTokenCount":1}}"#;
        assert_eq!(
            parse_response(AiProviderKind::Gemini, gemini).unwrap().text,
            "Hi"
        );

        let ollama =
            r#"{"message":{"content":"Local answer"},"prompt_eval_count":10,"eval_count":4}"#;
        assert_eq!(
            parse_response(AiProviderKind::Ollama, ollama).unwrap().text,
            "Local answer"
        );
    }

    #[test]
    fn rejects_empty_responses() {
        assert!(parse_response(AiProviderKind::OpenAi, r#"{"choices":[]}"#).is_err());
    }
}
