//! Provider-neutral batch transcription for the Bangla shortcut.
//!
//! The recording/action layer deals only in [`RecordedAudio`] and
//! [`BanglaTranscript`]. Provider wire formats, authentication, response
//! parsing, and provider-specific errors belong in their own adapter module.
//! This keeps local English transcription fully separate and makes a future
//! provider addition a contained change:
//!
//! 1. Add an adapter module implementing [`BanglaTranscriptionProvider`].
//! 2. Register it in [`transcribe_bangla`].
//! 3. Add its defaults and UI metadata in `settings.rs`.
//! 4. Add mock-server coverage for its request and response contract.
//!
//! Do not log audio samples, transcript text, API keys, or configured endpoint
//! URLs from this module or an adapter. Those values are private user data.

mod deepgram;

use crate::audio_toolkit::constants::WHISPER_SAMPLE_RATE;
use crate::settings::AppSettings;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

const MAX_BANGLA_AUDIO_SECONDS: usize = 9 * 60;
const MAX_BANGLA_AUDIO_SAMPLES: usize = WHISPER_SAMPLE_RATE as usize * MAX_BANGLA_AUDIO_SECONDS;

/// A completed capture ready for a batch provider. The recorder guarantees
/// 16 kHz mono samples; the provider owns its transport encoding.
#[derive(Debug, Clone)]
pub(crate) struct RecordedAudio {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
}

impl RecordedAudio {
    pub(crate) fn from_recorder(samples: Vec<f32>) -> Result<Self, CloudTranscriptionError> {
        if samples.is_empty() {
            return Err(CloudTranscriptionError::EmptyAudio);
        }
        if samples.len() > MAX_BANGLA_AUDIO_SAMPLES {
            return Err(CloudTranscriptionError::AudioTooLong);
        }

        Ok(Self {
            samples,
            sample_rate: WHISPER_SAMPLE_RATE,
            channels: 1,
        })
    }
}

/// Provider-independent, validated final result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BanglaTranscript {
    pub text: String,
}

impl BanglaTranscript {
    pub(crate) fn new(text: String) -> Result<Self, CloudTranscriptionError> {
        let text = text.trim().to_string();
        if text.is_empty() {
            return Err(CloudTranscriptionError::EmptyTranscript);
        }
        Ok(Self { text })
    }
}

/// A pull-based cancellation check shared by the action and a provider. The
/// action also drops the HTTP future promptly, which aborts the in-flight
/// request rather than merely suppressing its eventual result.
#[derive(Clone)]
pub(crate) struct CancellationContext(Arc<dyn Fn() -> bool + Send + Sync>);

impl CancellationContext {
    pub(crate) fn new(check: impl Fn() -> bool + Send + Sync + 'static) -> Self {
        Self(Arc::new(check))
    }

    pub(crate) fn is_cancelled(&self) -> bool {
        (self.0)()
    }
}

/// Errors deliberately distinguish user-actionable failure classes while
/// keeping provider response bodies and endpoint URLs out of logs and UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CloudTranscriptionError {
    Cancelled,
    MissingConfiguration,
    UnsupportedProvider,
    InvalidConfiguration,
    EmptyAudio,
    AudioTooLong,
    Offline,
    Timeout,
    Authentication,
    RateLimited,
    Provider,
    MalformedResponse,
    EmptyTranscript,
}

impl CloudTranscriptionError {
    /// Stable UI event code. The webview localizes this instead of receiving a
    /// raw transport error, which could contain sensitive configuration.
    pub(crate) fn event_code(self) -> &'static str {
        match self {
            Self::Cancelled => "cancelled",
            Self::MissingConfiguration => "missing_configuration",
            Self::UnsupportedProvider => "unsupported_provider",
            Self::InvalidConfiguration => "invalid_configuration",
            Self::EmptyAudio => "empty_audio",
            Self::AudioTooLong => "audio_too_long",
            Self::Offline => "offline",
            Self::Timeout => "timeout",
            Self::Authentication => "authentication",
            Self::RateLimited => "rate_limited",
            Self::Provider => "provider",
            Self::MalformedResponse => "malformed_response",
            Self::EmptyTranscript => "empty_transcript",
        }
    }
}

impl fmt::Display for CloudTranscriptionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.event_code())
    }
}

impl std::error::Error for CloudTranscriptionError {}

type TranscriptionFuture<'a> =
    Pin<Box<dyn Future<Output = Result<BanglaTranscript, CloudTranscriptionError>> + Send + 'a>>;

/// The only interface actions need to know. A provider adapter is responsible
/// for its own WAV/PCM conversion, HTTP protocol, and response shape.
pub(crate) trait BanglaTranscriptionProvider: Send + Sync {
    fn transcribe<'a>(
        &'a self,
        audio: RecordedAudio,
        settings: &'a AppSettings,
        cancellation: CancellationContext,
    ) -> TranscriptionFuture<'a>;
}

/// Dispatch the configured batch provider. This is intentionally separate from
/// local `TranscriptionManager`: the Bangla mode must never fall back to local
/// English inference when cloud configuration or a request fails.
pub(crate) async fn transcribe_bangla(
    audio: RecordedAudio,
    settings: &AppSettings,
    cancellation: CancellationContext,
) -> Result<BanglaTranscript, CloudTranscriptionError> {
    if cancellation.is_cancelled() {
        return Err(CloudTranscriptionError::Cancelled);
    }

    match settings.bangla_stt_provider_id.as_str() {
        deepgram::DEEPGRAM_PROVIDER_ID => {
            deepgram::DeepgramNova3Provider
                .transcribe(audio, settings, cancellation)
                .await
        }
        _ => Err(CloudTranscriptionError::UnsupportedProvider),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recorder_audio_is_limited_to_nine_minutes() {
        assert!(RecordedAudio::from_recorder(vec![0.0; MAX_BANGLA_AUDIO_SAMPLES]).is_ok());
        assert_eq!(
            RecordedAudio::from_recorder(vec![0.0; MAX_BANGLA_AUDIO_SAMPLES + 1]).unwrap_err(),
            CloudTranscriptionError::AudioTooLong
        );
    }

    #[test]
    fn transcripts_must_contain_text() {
        assert_eq!(
            BanglaTranscript::new("  \n\t ".to_string()).unwrap_err(),
            CloudTranscriptionError::EmptyTranscript
        );
    }
}
