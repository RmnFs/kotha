//! Required LLM Romanization for the Bangla shortcut.
//!
//! This service is deliberately separate from English post-processing. The
//! latter is optional and may retain a local transcript when it fails; this
//! module has an explicit success/error contract so the Bangla action can make
//! its user-visible fallback decision in one audited place.
//!
//! Do not log API keys, Bangla input, Romanized output, prompt content, raw
//! response bodies, or provider URLs from this module.

use crate::bangla_transcription::CancellationContext;
use crate::settings::AppSettings;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest::{StatusCode, Url};
use serde::Deserialize;
use serde_json::{json, Value};
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

pub(crate) const GROQ_PROVIDER_ID: &str = "groq";
pub(crate) const GEMINI_PROVIDER_ID: &str = "gemini";
pub(crate) const OPENAI_PROVIDER_ID: &str = "openai";

const ROMANIZATION_FIELD: &str = "romanized_text";
const GROQ_BASE_URL: &str = "https://api.groq.com/openai/v1";
const OPENAI_BASE_URL: &str = "https://api.openai.com/v1";
const GEMINI_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta";

/// A versioned, internal-only prompt. It is intentionally not shared with the
/// optional English post-processing prompt collection.
const ROMANIZATION_PROMPT_V1: &str = "";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RomanizationInput {
    pub bangla_text: String,
}

impl RomanizationInput {
    pub(crate) fn new(bangla_text: String) -> Result<Self, RomanizationError> {
        let bangla_text = bangla_text.trim().to_string();
        if bangla_text.is_empty() {
            return Err(RomanizationError::EmptyInput);
        }
        Ok(Self { bangla_text })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RomanizationResult {
    pub romanized_text: String,
}

impl RomanizationResult {
    fn new(romanized_text: String) -> Result<Self, RomanizationError> {
        let romanized_text = romanized_text.trim().to_string();
        if romanized_text.is_empty() {
            return Err(RomanizationError::EmptyResult);
        }
        // A mostly Romanized result may legitimately retain a few Bangla
        // characters. Accept the provider's non-empty output as requested.
        Ok(Self { romanized_text })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RomanizationError {
    Cancelled,
    MissingConfiguration,
    UnsupportedProvider,
    InvalidConfiguration,
    EmptyInput,
    Offline,
    Timeout,
    Authentication,
    RateLimited,
    Provider,
    MalformedResponse,
    EmptyResult,
}

impl RomanizationError {
    pub(crate) fn event_code(self) -> &'static str {
        match self {
            Self::Cancelled => "cancelled",
            Self::MissingConfiguration => "missing_configuration",
            Self::UnsupportedProvider => "unsupported_provider",
            Self::InvalidConfiguration => "invalid_configuration",
            Self::EmptyInput => "empty_input",
            Self::Offline => "offline",
            Self::Timeout => "timeout",
            Self::Authentication => "authentication",
            Self::RateLimited => "rate_limited",
            Self::Provider => "provider",
            Self::MalformedResponse => "malformed_response",
            Self::EmptyResult => "empty_result",
        }
    }
}

impl fmt::Display for RomanizationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.event_code())
    }
}

impl std::error::Error for RomanizationError {}

type RomanizationFuture<'a> =
    Pin<Box<dyn Future<Output = Result<RomanizationResult, RomanizationError>> + Send + 'a>>;

/// Provider-neutral boundary used by the recording/action pipeline.
pub(crate) trait BanglaRomanizationProvider: Send + Sync {
    fn romanize<'a>(
        &'a self,
        input: RomanizationInput,
        settings: &'a AppSettings,
        cancellation: CancellationContext,
    ) -> RomanizationFuture<'a>;
}

pub(crate) async fn romanize_bangla(
    input: RomanizationInput,
    settings: &AppSettings,
    cancellation: CancellationContext,
) -> Result<RomanizationResult, RomanizationError> {
    if cancellation.is_cancelled() {
        return Err(RomanizationError::Cancelled);
    }

    match settings.bangla_romanization_provider_id.as_str() {
        GROQ_PROVIDER_ID => {
            OpenAiCompatibleProvider {
                provider_id: GROQ_PROVIDER_ID,
                base_url: GROQ_BASE_URL,
            }
            .romanize(input, settings, cancellation)
            .await
        }
        OPENAI_PROVIDER_ID => {
            OpenAiCompatibleProvider {
                provider_id: OPENAI_PROVIDER_ID,
                base_url: OPENAI_BASE_URL,
            }
            .romanize(input, settings, cancellation)
            .await
        }
        GEMINI_PROVIDER_ID => GeminiProvider.romanize(input, settings, cancellation).await,
        _ => Err(RomanizationError::UnsupportedProvider),
    }
}

struct OpenAiCompatibleProvider {
    provider_id: &'static str,
    base_url: &'static str,
}

impl BanglaRomanizationProvider for OpenAiCompatibleProvider {
    fn romanize<'a>(
        &'a self,
        input: RomanizationInput,
        settings: &'a AppSettings,
        cancellation: CancellationContext,
    ) -> RomanizationFuture<'a> {
        Box::pin(async move {
            let (api_key, model, timeout) = provider_settings(settings, self.provider_id)?;
            let url = chat_completions_url(self.base_url)?;
            let client = bearer_client(api_key, timeout)?;
            let request = json!({
                "model": model,
                "stream": false,
                "response_format": { "type": "json_object" },
                "messages": [
                    { "role": "system", "content": ROMANIZATION_PROMPT_V1 },
                    { "role": "user", "content": input.bangla_text }
                ]
            });

            let response = client
                .post(url)
                .json(&request)
                .send()
                .await
                .map_err(map_request_error)?;
            if cancellation.is_cancelled() {
                return Err(RomanizationError::Cancelled);
            }
            if !response.status().is_success() {
                return Err(map_status(response.status()));
            }
            let response = response
                .json::<OpenAiCompatibleResponse>()
                .await
                .map_err(|_| RomanizationError::MalformedResponse)?;
            if cancellation.is_cancelled() {
                return Err(RomanizationError::Cancelled);
            }
            parse_json_result(
                response
                    .choices
                    .first()
                    .and_then(|choice| choice.message.content.as_deref()),
            )
        })
    }
}

struct GeminiProvider;

impl BanglaRomanizationProvider for GeminiProvider {
    fn romanize<'a>(
        &'a self,
        input: RomanizationInput,
        settings: &'a AppSettings,
        cancellation: CancellationContext,
    ) -> RomanizationFuture<'a> {
        Box::pin(async move {
            let (api_key, model, timeout) = provider_settings(settings, GEMINI_PROVIDER_ID)?;
            let url = gemini_url(&model)?;
            let client = gemini_client(api_key, timeout)?;
            let request = json!({
                "systemInstruction": { "parts": [{ "text": ROMANIZATION_PROMPT_V1 }] },
                "contents": [{ "role": "user", "parts": [{ "text": input.bangla_text }] }],
                "generationConfig": {
                    "responseMimeType": "application/json",
                    "responseJsonSchema": romanization_schema()
                }
            });

            let response = client
                .post(url)
                .json(&request)
                .send()
                .await
                .map_err(map_request_error)?;
            if cancellation.is_cancelled() {
                return Err(RomanizationError::Cancelled);
            }
            if !response.status().is_success() {
                return Err(map_status(response.status()));
            }
            let response = response
                .json::<GeminiResponse>()
                .await
                .map_err(|_| RomanizationError::MalformedResponse)?;
            if cancellation.is_cancelled() {
                return Err(RomanizationError::Cancelled);
            }
            parse_json_result(
                response
                    .candidates
                    .first()
                    .and_then(|candidate| candidate.content.parts.first())
                    .map(|part| part.text.as_str()),
            )
        })
    }
}

fn provider_settings<'a>(
    settings: &'a AppSettings,
    provider_id: &str,
) -> Result<(&'a str, &'a str, Duration), RomanizationError> {
    let api_key = settings
        .bangla_romanization_api_keys
        .get(provider_id)
        .map(String::as_str)
        .unwrap_or("")
        .trim();
    let model = settings
        .bangla_romanization_models
        .get(provider_id)
        .map(String::as_str)
        .unwrap_or("")
        .trim();
    if api_key.is_empty() || model.is_empty() {
        return Err(RomanizationError::MissingConfiguration);
    }
    if !(5..=120).contains(&settings.bangla_romanization_timeout_seconds) {
        return Err(RomanizationError::InvalidConfiguration);
    }
    let timeout = Duration::from_secs(settings.bangla_romanization_timeout_seconds);
    Ok((api_key, model, timeout))
}

fn bearer_client(api_key: &str, timeout: Duration) -> Result<reqwest::Client, RomanizationError> {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {api_key}"))
            .map_err(|_| RomanizationError::InvalidConfiguration)?,
    );
    reqwest::Client::builder()
        .default_headers(headers)
        .connect_timeout(timeout)
        .timeout(timeout)
        .build()
        .map_err(|_| RomanizationError::Provider)
}

fn gemini_client(api_key: &str, timeout: Duration) -> Result<reqwest::Client, RomanizationError> {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        "x-goog-api-key",
        HeaderValue::from_str(api_key).map_err(|_| RomanizationError::InvalidConfiguration)?,
    );
    reqwest::Client::builder()
        .default_headers(headers)
        .connect_timeout(timeout)
        .timeout(timeout)
        .build()
        .map_err(|_| RomanizationError::Provider)
}

fn chat_completions_url(base_url: &str) -> Result<Url, RomanizationError> {
    Url::parse(&format!(
        "{}/chat/completions",
        base_url.trim_end_matches('/')
    ))
    .map_err(|_| RomanizationError::InvalidConfiguration)
}

fn gemini_url(model: &str) -> Result<Url, RomanizationError> {
    let mut url =
        Url::parse(GEMINI_BASE_URL).map_err(|_| RomanizationError::InvalidConfiguration)?;
    url.path_segments_mut()
        .map_err(|_| RomanizationError::InvalidConfiguration)?
        .extend(["models", model]);
    let path = format!("{}:generateContent", url.path());
    url.set_path(&path);
    Ok(url)
}

fn romanization_schema() -> Value {
    json!({
        "type": "object",
        "properties": { ROMANIZATION_FIELD: { "type": "string" } },
        "required": [ROMANIZATION_FIELD],
        "additionalProperties": false
    })
}

fn parse_json_result(content: Option<&str>) -> Result<RomanizationResult, RomanizationError> {
    let content = content.ok_or(RomanizationError::MalformedResponse)?;
    let object =
        serde_json::from_str::<Value>(content).map_err(|_| RomanizationError::MalformedResponse)?;
    let text = object
        .get(ROMANIZATION_FIELD)
        .and_then(Value::as_str)
        .ok_or(RomanizationError::MalformedResponse)?;
    RomanizationResult::new(text.to_string())
}

fn map_request_error(error: reqwest::Error) -> RomanizationError {
    if error.is_timeout() {
        RomanizationError::Timeout
    } else if error.is_connect() {
        RomanizationError::Offline
    } else {
        RomanizationError::Provider
    }
}

fn map_status(status: StatusCode) -> RomanizationError {
    match status {
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => RomanizationError::Authentication,
        StatusCode::TOO_MANY_REQUESTS => RomanizationError::RateLimited,
        _ => RomanizationError::Provider,
    }
}

#[derive(Deserialize)]
struct OpenAiCompatibleResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessage,
}

#[derive(Deserialize)]
struct OpenAiMessage {
    content: Option<String>,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
}

#[derive(Deserialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Deserialize)]
struct GeminiPart {
    text: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::get_default_settings;

    #[test]
    fn accepts_non_empty_output_even_when_it_retains_bangla_characters() {
        let result = RomanizationResult::new("ami bhalo আছি".to_string()).unwrap();
        assert_eq!(result.romanized_text, "ami bhalo আছি");
    }

    #[test]
    fn results_require_a_non_empty_romanized_text_field() {
        assert_eq!(
            parse_json_result(Some(r#"{"romanized_text":" "}"#)).unwrap_err(),
            RomanizationError::EmptyResult
        );
        assert_eq!(
            parse_json_result(Some(r#"{"other":"value"}"#)).unwrap_err(),
            RomanizationError::MalformedResponse
        );
    }

    #[test]
    fn default_configuration_is_provider_scoped() {
        let settings = get_default_settings();
        for provider in [GROQ_PROVIDER_ID, GEMINI_PROVIDER_ID, OPENAI_PROVIDER_ID] {
            assert!(settings.bangla_romanization_api_keys.contains_key(provider));
            assert!(settings.bangla_romanization_models.contains_key(provider));
        }
    }

    #[test]
    fn actionable_statuses_are_mapped_without_response_bodies() {
        assert_eq!(
            map_status(StatusCode::UNAUTHORIZED),
            RomanizationError::Authentication
        );
        assert_eq!(
            map_status(StatusCode::TOO_MANY_REQUESTS),
            RomanizationError::RateLimited
        );
        assert_eq!(
            map_status(StatusCode::BAD_GATEWAY),
            RomanizationError::Provider
        );
    }
}
