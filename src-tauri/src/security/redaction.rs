use regex::Regex;
use std::sync::LazyLock;

static BEARER_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(bearer\s+)[A-Za-z0-9._~+/=-]+").unwrap_or_else(|_| unreachable!())
});
static KEY_VALUE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?i)(["']?(?:api[_-]?key|apikey|token|password|passphrase|secret|authorization|auth|credential)["']?\s*[:=]\s*)(["']?)([^"'\s,;}]+)"#,
    )
    .unwrap_or_else(|_| unreachable!())
});
static PREFIX_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)\b(?:sk-proj-|sk-|gh[pousr]_|github_pat_|xox|AIza|ya29\.|eyJ)[A-Za-z0-9._~+/=-]+",
    )
    .unwrap_or_else(|_| unreachable!())
});

pub fn redact(input: &str) -> String {
    let bearer = BEARER_PATTERN.replace_all(input, "$1[REDACTED]");
    let key_value = KEY_VALUE_PATTERN.replace_all(&bearer, |captures: &regex::Captures| {
        let quote = &captures[2];
        format!("{}{}[REDACTED]{quote}", &captures[1], quote)
    });
    PREFIX_PATTERN
        .replace_all(&key_value, "[REDACTED]")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redacts_key_value_credentials() {
        let redacted = redact("api_key=sk-abc123 token: secret-value");
        assert!(!redacted.contains("sk-abc123"));
        assert!(!redacted.contains("secret-value"));
        assert!(redacted.contains("[REDACTED]"));
    }

    #[test]
    fn redacts_bearer_tokens() {
        let redacted = redact("Authorization: Bearer eyJhbGciOi.payload.signature");
        assert!(!redacted.contains("eyJhbGciOi"));
        assert!(!redacted.contains("payload.signature"));
    }

    #[test]
    fn preserves_non_secret_text() {
        let line = "Refreshed 12 feeds in 430ms";
        assert_eq!(redact(line), line);
    }

    #[test]
    fn redacts_json_style_secrets() {
        let redacted = redact(r#"{"type":"openai","api_key":"secret-xyz"}"#);
        assert!(!redacted.contains("secret-xyz"));
    }

    #[test]
    fn redacts_github_tokens() {
        let redacted = redact("token=ghp_1234567890abcdef");
        assert!(!redacted.contains("ghp_1234567890abcdef"));
    }
}
