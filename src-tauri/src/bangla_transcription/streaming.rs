//! Lifecycle and audio routing for optional Bangla cloud streaming.
//!
//! The recorder calls [`BanglaStreamingManager::feed`] from its capture worker.
//! That method must stay non-blocking: all WebSocket, PCM conversion, response
//! parsing, and finalization work happens in the async Deepgram worker.

use super::deepgram::DEEPGRAM_PROVIDER_ID;
use super::deepgram_streaming;
use super::{BanglaTranscript, CloudTranscriptionError, MAX_BANGLA_AUDIO_SAMPLES};
use crate::settings::AppSettings;
use log::debug;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot, watch};

/// Roughly fifteen seconds of 30 ms audio frames. This absorbs connection
/// setup jitter without allowing an offline endpoint to grow memory without
/// bound. A full local recording is retained independently for batch fallback.
const AUDIO_QUEUE_CAPACITY: usize = 512;
const SESSION_RESULT_TIMEOUT: Duration = Duration::from_secs(35);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AbortReason {
    Cancelled,
    QueueOverflow,
    AudioTooLong,
}

pub(super) enum StreamCommand {
    Audio(Vec<f32>),
    Finish,
}

/// A content-free streaming failure. `fallback_allowed` prevents a second
/// request when batch cannot plausibly help (bad credentials, invalid config,
/// cancellation, rate limiting, or a recording beyond the product limit).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct StreamingFailure {
    pub error: CloudTranscriptionError,
    pub fallback_allowed: bool,
}

impl StreamingFailure {
    pub(super) fn recoverable(error: CloudTranscriptionError) -> Self {
        Self {
            error,
            fallback_allowed: true,
        }
    }

    pub(super) fn terminal(error: CloudTranscriptionError) -> Self {
        Self {
            error,
            fallback_allowed: false,
        }
    }

    fn from_abort(reason: AbortReason) -> Self {
        match reason {
            AbortReason::Cancelled => Self::terminal(CloudTranscriptionError::Cancelled),
            AbortReason::QueueOverflow => Self::recoverable(CloudTranscriptionError::Provider),
            AbortReason::AudioTooLong => Self::terminal(CloudTranscriptionError::AudioTooLong),
        }
    }
}

struct RunningSession {
    commands: mpsc::Sender<StreamCommand>,
    abort: watch::Sender<Option<AbortReason>>,
    result: oneshot::Receiver<Result<BanglaTranscript, StreamingFailure>>,
    accepted_samples: usize,
}

enum ActiveSession {
    Running(RunningSession),
    Failed(StreamingFailure),
}

/// Owns at most one Bangla streaming attempt. The transcription coordinator
/// already enforces one active recording, and this manager makes the same
/// invariant explicit at the cloud boundary.
pub(crate) struct BanglaStreamingManager {
    active: Mutex<Option<ActiveSession>>,
    abort: Mutex<Option<watch::Sender<Option<AbortReason>>>>,
    accepting_audio: AtomicBool,
}

impl BanglaStreamingManager {
    pub(crate) fn new() -> Self {
        Self {
            active: Mutex::new(None),
            abort: Mutex::new(None),
            accepting_audio: AtomicBool::new(false),
        }
    }

    /// Start a provider session before capture begins. Configuration failures
    /// are retained as a terminal result so stop follows the normal error path
    /// without exposing credentials or configuration values.
    pub(crate) fn start(&self, settings: &AppSettings) {
        self.cancel_active();

        if settings.bangla_stt_provider_id != DEEPGRAM_PROVIDER_ID {
            *self.active.lock().unwrap() = Some(ActiveSession::Failed(StreamingFailure::terminal(
                CloudTranscriptionError::UnsupportedProvider,
            )));
            return;
        }

        let config = match deepgram_streaming::DeepgramStreamingConfig::from_settings(settings) {
            Ok(config) => config,
            Err(failure) => {
                *self.active.lock().unwrap() = Some(ActiveSession::Failed(failure));
                return;
            }
        };

        let (command_tx, command_rx) = mpsc::channel(AUDIO_QUEUE_CAPACITY);
        let (abort_tx, abort_rx) = watch::channel(None);
        let (result_tx, result_rx) = oneshot::channel();
        tauri::async_runtime::spawn(async move {
            let result = deepgram_streaming::run(config, command_rx, abort_rx).await;
            let _ = result_tx.send(result);
        });

        *self.active.lock().unwrap() = Some(ActiveSession::Running(RunningSession {
            commands: command_tx,
            abort: abort_tx.clone(),
            result: result_rx,
            accepted_samples: 0,
        }));
        *self.abort.lock().unwrap() = Some(abort_tx);
        self.accepting_audio.store(true, Ordering::Release);
    }

    /// Forward one 16 kHz mono frame that passed the configured capture policy
    /// without ever waiting for the network. On overflow the streaming attempt
    /// is abandoned and the caller can use the independently retained batch
    /// recording after release.
    pub(crate) fn feed(&self, frame: &[f32]) {
        if !self.accepting_audio.load(Ordering::Acquire) {
            return;
        }

        let mut active = self.active.lock().unwrap();
        if !self.accepting_audio.load(Ordering::Acquire) {
            return;
        }
        let Some(ActiveSession::Running(session)) = active.as_mut() else {
            self.accepting_audio.store(false, Ordering::Release);
            return;
        };

        let Some(next_sample_count) = session.accepted_samples.checked_add(frame.len()) else {
            Self::abort_running(session, AbortReason::AudioTooLong);
            self.accepting_audio.store(false, Ordering::Release);
            return;
        };
        if next_sample_count > MAX_BANGLA_AUDIO_SAMPLES {
            Self::abort_running(session, AbortReason::AudioTooLong);
            self.accepting_audio.store(false, Ordering::Release);
            return;
        }

        match session
            .commands
            .try_send(StreamCommand::Audio(frame.to_vec()))
        {
            Ok(()) => session.accepted_samples = next_sample_count,
            Err(mpsc::error::TrySendError::Full(_)) => {
                Self::abort_running(session, AbortReason::QueueOverflow);
                self.accepting_audio.store(false, Ordering::Release);
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                self.accepting_audio.store(false, Ordering::Release);
            }
        }
    }

    /// Stop accepting frames, enqueue finalization after every prior audio
    /// command, and wait for the provider's complete final transcript.
    /// `None` means this recording selected batch mode and opened no session.
    pub(crate) async fn finish(&self) -> Option<Result<BanglaTranscript, StreamingFailure>> {
        self.accepting_audio.store(false, Ordering::Release);
        let active = self.active.lock().unwrap().take()?;

        let result = match active {
            ActiveSession::Failed(failure) => Some(Err(failure)),
            ActiveSession::Running(session) => {
                let abort_reason = *session.abort.borrow();
                if let Some(reason) = abort_reason {
                    Some(Err(StreamingFailure::from_abort(reason)))
                } else {
                    if session.commands.send(StreamCommand::Finish).await.is_err() {
                        debug!("Bangla streaming worker ended before finalization was queued");
                    }

                    match tokio::time::timeout(SESSION_RESULT_TIMEOUT, session.result).await {
                        Ok(Ok(result)) => Some(result),
                        Ok(Err(_)) => Some(Err(StreamingFailure::recoverable(
                            CloudTranscriptionError::Provider,
                        ))),
                        Err(_) => {
                            let _ = session.abort.send(Some(AbortReason::Cancelled));
                            Some(Err(StreamingFailure::recoverable(
                                CloudTranscriptionError::Timeout,
                            )))
                        }
                    }
                }
            }
        };
        self.abort.lock().unwrap().take();
        result
    }

    /// Immediately abandon an active stream. Already transmitted audio cannot
    /// be recalled, but no final result, batch fallback, Romanization, or paste
    /// will be produced by the cancelled operation.
    pub(crate) fn cancel_active(&self) {
        self.accepting_audio.store(false, Ordering::Release);
        if let Some(abort) = self.abort.lock().unwrap().take() {
            let _ = abort.send(Some(AbortReason::Cancelled));
        }
        if let Some(ActiveSession::Running(session)) = self.active.lock().unwrap().take() {
            let _ = session.abort.send(Some(AbortReason::Cancelled));
        }
    }

    fn abort_running(session: &RunningSession, reason: AbortReason) {
        if session.abort.borrow().is_none() {
            let _ = session.abort.send(Some(reason));
        }
    }
}

impl Default for BanglaStreamingManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::get_default_settings;

    #[tokio::test]
    async fn an_unstarted_manager_represents_batch_mode() {
        let manager = BanglaStreamingManager::new();
        assert_eq!(manager.finish().await, None);
    }

    #[tokio::test]
    async fn missing_credentials_fail_without_a_batch_retry() {
        let manager = BanglaStreamingManager::new();
        manager.start(&get_default_settings());

        let failure = manager
            .finish()
            .await
            .expect("streaming was selected")
            .expect_err("missing credentials must fail");
        assert_eq!(failure.error, CloudTranscriptionError::MissingConfiguration);
        assert!(!failure.fallback_allowed);
    }
}
