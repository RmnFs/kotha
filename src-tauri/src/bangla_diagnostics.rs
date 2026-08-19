//! Session-only, content-free diagnostics for the Bangla cloud pipeline.
//!
//! The latest snapshot is kept in process memory only. This module must never
//! receive audio, transcript text, Romanized text, prompts, credentials,
//! endpoints, authorization headers, or raw provider responses.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::Mutex;
use tauri::AppHandle;
use tauri_specta::Event;

use crate::settings::get_settings;

static LATEST_BANGLA_DIAGNOSTIC: Lazy<Mutex<Option<BanglaDiagnostic>>> =
    Lazy::new(|| Mutex::new(None));

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum BanglaDiagnosticOutcomeCategory {
    Romanized,
    RawBangla,
    RomanizationFallback,
    Cancelled,
    Failed,
}

/// A privacy-safe snapshot of one terminal Bangla operation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct BanglaDiagnostic {
    pub outcome: String,
    pub outcome_category: BanglaDiagnosticOutcomeCategory,
    pub error_code: Option<String>,
    pub fallback_reason: Option<String>,
    pub stt_provider: String,
    pub stt_model: String,
    pub stt_transport: String,
    pub romanization_enabled: bool,
    pub romanization_provider: Option<String>,
    pub romanization_model: Option<String>,
    pub recording_duration_ms: u64,
    pub recorder_stop_ms: u64,
    pub stt_finalize_ms: Option<u64>,
    pub stt_ms: u64,
    pub romanization_ms: u64,
    pub romanization_headers_ms: Option<u64>,
    pub romanization_body_ms: Option<u64>,
    pub provider_queue_ms: Option<u64>,
    pub provider_prompt_ms: Option<u64>,
    pub provider_completion_ms: Option<u64>,
    pub provider_total_ms: Option<u64>,
    pub provider_prompt_tokens: Option<u64>,
    pub provider_output_tokens: Option<u64>,
    pub provider_thinking_tokens: Option<u64>,
    pub provider_request_id: Option<String>,
    pub paste_queue_ms: u64,
    pub paste_call_ms: u64,
    pub post_stop_total_ms: u64,
    pub recording_to_terminal_ms: u64,
}

/// `diagnostic: null` tells an open Bangla page to clear its session view.
#[derive(Clone, Debug, Serialize, Deserialize, Type, tauri_specta::Event)]
pub struct BanglaDiagnosticEvent {
    pub diagnostic: Option<BanglaDiagnostic>,
}

fn with_latest<T>(operation: impl FnOnce(&mut Option<BanglaDiagnostic>) -> T) -> T {
    let mut latest = LATEST_BANGLA_DIAGNOSTIC
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    operation(&mut latest)
}

pub(crate) fn publish(app: &AppHandle, diagnostic: BanglaDiagnostic) {
    // A request may have started while Debug Mode was enabled and finished
    // after it was disabled. Recheck here so a late result cannot repopulate
    // the session snapshot after the user turns diagnostics off.
    if !get_settings(app).debug_mode {
        return;
    }

    with_latest(|latest| *latest = Some(diagnostic.clone()));
    if let Err(error) = (BanglaDiagnosticEvent {
        diagnostic: Some(diagnostic),
    })
    .emit(app)
    {
        log::debug!("Failed to emit Bangla diagnostic event: {error}");
    }
}

pub(crate) fn clear(app: &AppHandle) {
    with_latest(|latest| *latest = None);
    if let Err(error) = (BanglaDiagnosticEvent { diagnostic: None }).emit(app) {
        log::debug!("Failed to emit Bangla diagnostic clear event: {error}");
    }
}

#[tauri::command]
#[specta::specta]
pub fn get_latest_bangla_diagnostic(app: AppHandle) -> Result<Option<BanglaDiagnostic>, String> {
    if !get_settings(&app).debug_mode {
        return Ok(None);
    }
    Ok(with_latest(|latest| latest.clone()))
}

#[tauri::command]
#[specta::specta]
pub fn clear_latest_bangla_diagnostic(app: AppHandle) -> Result<(), String> {
    clear(&app);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn example_diagnostic() -> BanglaDiagnostic {
        BanglaDiagnostic {
            outcome: "pasted_romanized".to_string(),
            outcome_category: BanglaDiagnosticOutcomeCategory::Romanized,
            error_code: None,
            fallback_reason: None,
            stt_provider: "deepgram".to_string(),
            stt_model: "nova-3".to_string(),
            stt_transport: "streaming".to_string(),
            romanization_enabled: true,
            romanization_provider: Some("groq".to_string()),
            romanization_model: Some("openai/gpt-oss-120b".to_string()),
            recording_duration_ms: 1_000,
            recorder_stop_ms: 12,
            stt_finalize_ms: Some(220),
            stt_ms: 220,
            romanization_ms: 410,
            romanization_headers_ms: Some(300),
            romanization_body_ms: Some(110),
            provider_queue_ms: Some(10),
            provider_prompt_ms: Some(20),
            provider_completion_ms: Some(250),
            provider_total_ms: Some(280),
            provider_prompt_tokens: Some(100),
            provider_output_tokens: Some(20),
            provider_thinking_tokens: None,
            provider_request_id: Some("request-id".to_string()),
            paste_queue_ms: 3,
            paste_call_ms: 4,
            post_stop_total_ms: 649,
            recording_to_terminal_ms: 1_649,
        }
    }

    #[test]
    fn serialized_diagnostic_has_an_explicit_content_free_allowlist() {
        let value = serde_json::to_value(example_diagnostic()).unwrap();
        let keys = value
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>();
        let expected = [
            "error_code",
            "fallback_reason",
            "outcome",
            "outcome_category",
            "paste_call_ms",
            "paste_queue_ms",
            "post_stop_total_ms",
            "provider_completion_ms",
            "provider_output_tokens",
            "provider_prompt_ms",
            "provider_prompt_tokens",
            "provider_queue_ms",
            "provider_request_id",
            "provider_thinking_tokens",
            "provider_total_ms",
            "recording_duration_ms",
            "recording_to_terminal_ms",
            "recorder_stop_ms",
            "romanization_body_ms",
            "romanization_enabled",
            "romanization_headers_ms",
            "romanization_model",
            "romanization_ms",
            "romanization_provider",
            "stt_finalize_ms",
            "stt_model",
            "stt_ms",
            "stt_provider",
            "stt_transport",
        ]
        .into_iter()
        .map(str::to_string)
        .collect::<BTreeSet<_>>();

        assert_eq!(keys, expected);
        for forbidden in [
            "audio",
            "transcript",
            "romanized_text",
            "prompt",
            "api_key",
            "endpoint",
            "authorization",
            "response_body",
        ] {
            assert!(!keys.contains(forbidden));
        }
    }

    #[test]
    fn event_can_explicitly_clear_the_frontend_snapshot() {
        let value = serde_json::to_value(BanglaDiagnosticEvent { diagnostic: None }).unwrap();
        assert!(value["diagnostic"].is_null());
    }
}
