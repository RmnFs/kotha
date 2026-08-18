//! Deepgram Nova-3 WebSocket adapter for the optional Bangla streaming mode.
//!
//! This module never logs endpoint URLs, authorization data, audio, transcript
//! text, response bodies, or WebSocket close descriptions.

use super::deepgram::DEEPGRAM_PROVIDER_ID;
use super::streaming::{AbortReason, StreamCommand, StreamingFailure};
use super::{BanglaTranscript, CloudTranscriptionError};
use crate::settings::AppSettings;
use futures_util::{SinkExt, StreamExt};
use reqwest::Url;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::future::pending;
use std::time::Duration;
use tokio::sync::{mpsc, watch};
use tokio::time::{Instant, MissedTickBehavior};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::header::AUTHORIZATION;
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::tungstenite::{Error as WebSocketError, Message};

const DEFAULT_DEEPGRAM_ENDPOINT: &str = "https://api.deepgram.com/v1/listen";
const DEFAULT_DEEPGRAM_MODEL: &str = "nova-3";
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const FINALIZE_TIMEOUT: Duration = Duration::from_secs(20);
const KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(4);

pub(super) struct DeepgramStreamingConfig {
    url: Url,
    api_key: String,
}

impl DeepgramStreamingConfig {
    pub(super) fn from_settings(settings: &AppSettings) -> Result<Self, StreamingFailure> {
        let api_key = settings
            .bangla_stt_api_keys
            .get(DEEPGRAM_PROVIDER_ID)
            .map(String::as_str)
            .unwrap_or("")
            .trim();
        if api_key.is_empty() {
            return Err(StreamingFailure::terminal(
                CloudTranscriptionError::MissingConfiguration,
            ));
        }

        let configured_endpoint = settings.bangla_stt_endpoint.trim();
        let endpoint = if configured_endpoint.is_empty() {
            DEFAULT_DEEPGRAM_ENDPOINT
        } else {
            configured_endpoint
        };
        let model = settings
            .bangla_stt_models
            .get(DEEPGRAM_PROVIDER_ID)
            .map(String::as_str)
            .unwrap_or(DEFAULT_DEEPGRAM_MODEL)
            .trim();
        if model.is_empty() {
            return Err(StreamingFailure::terminal(
                CloudTranscriptionError::MissingConfiguration,
            ));
        }

        Ok(Self {
            url: streaming_url(endpoint, model)?,
            api_key: api_key.to_string(),
        })
    }
}

fn streaming_url(endpoint: &str, model: &str) -> Result<Url, StreamingFailure> {
    let mut url = Url::parse(endpoint)
        .map_err(|_| StreamingFailure::terminal(CloudTranscriptionError::InvalidConfiguration))?;
    let websocket_scheme = match url.scheme() {
        "https" | "wss" => "wss",
        "http" | "ws" => "ws",
        _ => {
            return Err(StreamingFailure::terminal(
                CloudTranscriptionError::InvalidConfiguration,
            ))
        }
    };
    url.set_scheme(websocket_scheme)
        .map_err(|_| StreamingFailure::terminal(CloudTranscriptionError::InvalidConfiguration))?;
    {
        let mut query = url.query_pairs_mut();
        query.append_pair("model", model);
        query.append_pair("language", "bn");
        query.append_pair("smart_format", "true");
        query.append_pair("encoding", "linear16");
        query.append_pair("sample_rate", "16000");
        query.append_pair("channels", "1");
    }
    Ok(url)
}

pub(super) async fn run(
    config: DeepgramStreamingConfig,
    mut commands: mpsc::Receiver<StreamCommand>,
    mut abort: watch::Receiver<Option<AbortReason>>,
) -> Result<BanglaTranscript, StreamingFailure> {
    let mut request =
        config.url.as_str().into_client_request().map_err(|_| {
            StreamingFailure::terminal(CloudTranscriptionError::InvalidConfiguration)
        })?;
    let authorization = HeaderValue::from_str(&format!("Token {}", config.api_key))
        .map_err(|_| StreamingFailure::terminal(CloudTranscriptionError::InvalidConfiguration))?;
    request.headers_mut().insert(AUTHORIZATION, authorization);

    let connect = tokio_tungstenite::connect_async(request);
    let (socket, _) = tokio::select! {
        reason = wait_for_abort(&mut abort) => return Err(failure_from_abort(reason)),
        result = tokio::time::timeout(CONNECT_TIMEOUT, connect) => {
            match result {
                Ok(Ok(connection)) => connection,
                Ok(Err(error)) => return Err(map_websocket_error(error)),
                Err(_) => return Err(StreamingFailure::recoverable(CloudTranscriptionError::Timeout)),
            }
        }
    };

    let (mut writer, mut reader) = socket.split();
    let mut keep_alive = tokio::time::interval(KEEP_ALIVE_INTERVAL);
    keep_alive.set_missed_tick_behavior(MissedTickBehavior::Skip);
    keep_alive.tick().await;

    let mut transcript = TranscriptAccumulator::default();
    let mut finalize_deadline = None;

    loop {
        tokio::select! {
            reason = wait_for_abort(&mut abort) => return Err(failure_from_abort(reason)),
            command = commands.recv(), if finalize_deadline.is_none() => {
                match command {
                    Some(StreamCommand::Audio(samples)) => {
                        writer
                            .send(Message::Binary(encode_linear16(&samples).into()))
                            .await
                            .map_err(map_websocket_error)?;
                    }
                    Some(StreamCommand::Finish) => {
                        // CloseStream is Deepgram's terminal flush: it processes
                        // cached audio, returns final Results + Metadata, then
                        // terminates the server side of the connection.
                        writer
                            .send(Message::Text(r#"{"type":"CloseStream"}"#.into()))
                            .await
                            .map_err(map_websocket_error)?;
                        finalize_deadline = Some(Instant::now() + FINALIZE_TIMEOUT);
                    }
                    None => {
                        return Err(StreamingFailure::recoverable(
                            CloudTranscriptionError::Provider,
                        ));
                    }
                }
            }
            _ = keep_alive.tick(), if finalize_deadline.is_none() => {
                writer
                    .send(Message::Text(r#"{"type":"KeepAlive"}"#.into()))
                    .await
                    .map_err(map_websocket_error)?;
            }
            message = reader.next() => {
                match message {
                    Some(Ok(Message::Text(text))) => {
                        if consume_response(text.as_ref(), &mut transcript)?
                            && finalize_deadline.is_some()
                        {
                            return transcript.finish();
                        }
                    }
                    Some(Ok(Message::Ping(payload))) => {
                        writer
                            .send(Message::Pong(payload))
                            .await
                            .map_err(map_websocket_error)?;
                    }
                    Some(Ok(Message::Close(_))) | None if finalize_deadline.is_some() => {
                        return transcript.finish();
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        return Err(StreamingFailure::recoverable(
                            CloudTranscriptionError::Provider,
                        ));
                    }
                    Some(Ok(_)) => {}
                    Some(Err(error)) => return Err(map_websocket_error(error)),
                }
            }
            _ = wait_for_deadline(finalize_deadline) => {
                return Err(StreamingFailure::recoverable(
                    CloudTranscriptionError::Timeout,
                ));
            }
        }
    }
}

async fn wait_for_abort(abort: &mut watch::Receiver<Option<AbortReason>>) -> AbortReason {
    loop {
        if let Some(reason) = *abort.borrow() {
            return reason;
        }
        if abort.changed().await.is_err() {
            return AbortReason::Cancelled;
        }
    }
}

async fn wait_for_deadline(deadline: Option<Instant>) {
    match deadline {
        Some(deadline) => tokio::time::sleep_until(deadline).await,
        None => pending::<()>().await,
    }
}

fn failure_from_abort(reason: AbortReason) -> StreamingFailure {
    match reason {
        AbortReason::Cancelled => StreamingFailure::terminal(CloudTranscriptionError::Cancelled),
        AbortReason::QueueOverflow => {
            StreamingFailure::recoverable(CloudTranscriptionError::Provider)
        }
        AbortReason::AudioTooLong => {
            StreamingFailure::terminal(CloudTranscriptionError::AudioTooLong)
        }
    }
}

fn map_websocket_error(error: WebSocketError) -> StreamingFailure {
    match error {
        WebSocketError::Http(response) => match response.status().as_u16() {
            401 | 403 => StreamingFailure::terminal(CloudTranscriptionError::Authentication),
            429 => StreamingFailure::terminal(CloudTranscriptionError::RateLimited),
            400 => StreamingFailure::terminal(CloudTranscriptionError::InvalidConfiguration),
            404 | 405 => StreamingFailure::recoverable(CloudTranscriptionError::Provider),
            status if status >= 500 => {
                StreamingFailure::recoverable(CloudTranscriptionError::Provider)
            }
            _ => StreamingFailure::recoverable(CloudTranscriptionError::Provider),
        },
        WebSocketError::Io(_) | WebSocketError::Tls(_) => {
            StreamingFailure::recoverable(CloudTranscriptionError::Offline)
        }
        WebSocketError::Url(_) | WebSocketError::HttpFormat(_) => {
            StreamingFailure::terminal(CloudTranscriptionError::InvalidConfiguration)
        }
        _ => StreamingFailure::recoverable(CloudTranscriptionError::Provider),
    }
}

fn encode_linear16(samples: &[f32]) -> Vec<u8> {
    let mut pcm = Vec::with_capacity(samples.len() * 2);
    for sample in samples {
        let value = (sample.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16;
        pcm.extend_from_slice(&value.to_le_bytes());
    }
    pcm
}

#[derive(Default)]
struct TranscriptAccumulator {
    segments: BTreeMap<(u64, u64), String>,
}

impl TranscriptAccumulator {
    fn insert(&mut self, result: DeepgramStreamingResult) -> Result<(), StreamingFailure> {
        if !result.is_final {
            return Ok(());
        }
        if !result.start.is_finite()
            || result.start < 0.0
            || !result.duration.is_finite()
            || result.duration < 0.0
        {
            return Err(StreamingFailure::recoverable(
                CloudTranscriptionError::MalformedResponse,
            ));
        }

        let Some(alternative) = result.channel.alternatives.into_iter().next() else {
            return Err(StreamingFailure::recoverable(
                CloudTranscriptionError::MalformedResponse,
            ));
        };
        let text = alternative.transcript.trim();
        if text.is_empty() {
            return Ok(());
        }

        let start_micros = (result.start * 1_000_000.0).round() as u64;
        let duration_micros = (result.duration * 1_000_000.0).round() as u64;
        self.segments
            .insert((start_micros, duration_micros), text.to_string());
        Ok(())
    }

    fn finish(self) -> Result<BanglaTranscript, StreamingFailure> {
        let text = self.segments.into_values().collect::<Vec<_>>().join(" ");
        BanglaTranscript::new(text).map_err(StreamingFailure::recoverable)
    }
}

fn consume_response(
    text: &str,
    transcript: &mut TranscriptAccumulator,
) -> Result<bool, StreamingFailure> {
    let value: serde_json::Value = serde_json::from_str(text)
        .map_err(|_| StreamingFailure::recoverable(CloudTranscriptionError::MalformedResponse))?;
    match value.get("type").and_then(serde_json::Value::as_str) {
        Some("Results") => {
            let result =
                serde_json::from_value::<DeepgramStreamingResult>(value).map_err(|_| {
                    StreamingFailure::recoverable(CloudTranscriptionError::MalformedResponse)
                })?;
            transcript.insert(result)?;
            Ok(false)
        }
        Some("Metadata") => Ok(true),
        Some("Error") => Err(map_provider_error_code(
            value
                .get("code")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default(),
        )),
        Some(_) => Ok(false),
        None => Err(StreamingFailure::recoverable(
            CloudTranscriptionError::MalformedResponse,
        )),
    }
}

fn map_provider_error_code(code: &str) -> StreamingFailure {
    let normalized = code.to_ascii_lowercase();
    if normalized.contains("auth") || normalized.contains("token") {
        StreamingFailure::terminal(CloudTranscriptionError::Authentication)
    } else if normalized.contains("rate") || normalized.contains("limit") {
        StreamingFailure::terminal(CloudTranscriptionError::RateLimited)
    } else if normalized.contains("invalid") || normalized.contains("data") {
        StreamingFailure::terminal(CloudTranscriptionError::InvalidConfiguration)
    } else {
        StreamingFailure::recoverable(CloudTranscriptionError::Provider)
    }
}

#[derive(Deserialize)]
struct DeepgramStreamingResult {
    start: f64,
    duration: f64,
    is_final: bool,
    channel: DeepgramStreamingChannel,
}

#[derive(Deserialize)]
struct DeepgramStreamingChannel {
    alternatives: Vec<DeepgramStreamingAlternative>,
}

#[derive(Deserialize)]
struct DeepgramStreamingAlternative {
    transcript: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::{SinkExt, StreamExt};
    use tokio::net::TcpListener;
    use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};

    #[test]
    fn streaming_url_uses_raw_16khz_bangla_audio() {
        let url = streaming_url("https://api.deepgram.com/v1/listen", "nova-3").unwrap();
        assert_eq!(url.scheme(), "wss");
        let query: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(query["model"], "nova-3");
        assert_eq!(query["language"], "bn");
        assert_eq!(query["encoding"], "linear16");
        assert_eq!(query["sample_rate"], "16000");
        assert_eq!(query["channels"], "1");
        assert_eq!(query["smart_format"], "true");
    }

    #[test]
    fn custom_http_endpoint_becomes_ws() {
        let url = streaming_url("http://127.0.0.1:1234/v1/listen", "nova-3").unwrap();
        assert_eq!(url.scheme(), "ws");
    }

    #[test]
    fn linear16_encoding_is_little_endian_and_clamped() {
        assert_eq!(
            encode_linear16(&[0.0, 1.0, -1.0, 2.0]),
            [0i16, i16::MAX, -i16::MAX, i16::MAX]
                .into_iter()
                .flat_map(i16::to_le_bytes)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn only_final_segments_are_ordered_and_deduplicated() {
        let mut transcript = TranscriptAccumulator::default();
        consume_response(
            r#"{"type":"Results","start":1.0,"duration":1.0,"is_final":true,"channel":{"alternatives":[{"transcript":"আছি"}]}}"#,
            &mut transcript,
        )
        .unwrap();
        consume_response(
            r#"{"type":"Results","start":0.0,"duration":1.0,"is_final":false,"channel":{"alternatives":[{"transcript":"draft"}]}}"#,
            &mut transcript,
        )
        .unwrap();
        consume_response(
            r#"{"type":"Results","start":0.0,"duration":1.0,"is_final":true,"channel":{"alternatives":[{"transcript":"আমি"}]}}"#,
            &mut transcript,
        )
        .unwrap();
        consume_response(
            r#"{"type":"Results","start":1.0,"duration":1.0,"is_final":true,"channel":{"alternatives":[{"transcript":"ভালো আছি"}]}}"#,
            &mut transcript,
        )
        .unwrap();

        assert_eq!(transcript.finish().unwrap().text, "আমি ভালো আছি");
    }

    #[test]
    fn empty_final_stream_is_recoverable_for_batch_fallback() {
        let failure = TranscriptAccumulator::default().finish().unwrap_err();
        assert_eq!(failure.error, CloudTranscriptionError::EmptyTranscript);
        assert!(failure.fallback_allowed);
    }

    #[test]
    fn provider_auth_and_rate_errors_never_trigger_a_second_request() {
        for code in ["INVALID_AUTH", "TOKEN_EXPIRED", "RATE_LIMITED"] {
            let failure = map_provider_error_code(code);
            assert!(!failure.fallback_allowed, "code={code}");
        }

        let failure = map_provider_error_code("INTERNAL_SERVER_ERROR");
        assert!(failure.fallback_allowed);
    }

    #[allow(clippy::result_large_err)]
    #[tokio::test]
    async fn websocket_stream_sends_audio_then_close_and_collects_final_results() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (socket, _) = listener.accept().await.unwrap();
            let mut websocket = tokio_tungstenite::accept_hdr_async(
                socket,
                |request: &Request, response: Response| {
                    assert_eq!(
                        request.headers()[AUTHORIZATION],
                        HeaderValue::from_static("Token test-key")
                    );
                    let query = request.uri().query().unwrap();
                    assert!(query.contains("model=nova-3"));
                    assert!(query.contains("language=bn"));
                    assert!(query.contains("encoding=linear16"));
                    assert!(query.contains("sample_rate=16000"));
                    Ok(response)
                },
            )
            .await
            .unwrap();

            let mut received_audio = Vec::new();
            loop {
                match websocket.next().await.unwrap().unwrap() {
                    Message::Binary(bytes) => received_audio.extend_from_slice(&bytes),
                    Message::Text(text) if text.contains("CloseStream") => break,
                    Message::Text(text) if text.contains("KeepAlive") => {}
                    other => panic!("unexpected client message: {other:?}"),
                }
            }
            assert_eq!(received_audio, encode_linear16(&[0.0, 1.0, -1.0]));

            websocket
                .send(Message::Text(
                    r#"{"type":"Results","start":0.0,"duration":1.0,"is_final":true,"channel":{"alternatives":[{"transcript":"আমি ভালো আছি"}]}}"#
                        .into(),
                ))
                .await
                .unwrap();
            websocket
                .send(Message::Text(
                    r#"{"type":"Metadata","request_id":"safe-test-id"}"#.into(),
                ))
                .await
                .unwrap();
        });

        let config = DeepgramStreamingConfig {
            url: streaming_url(&format!("http://{address}/v1/listen"), "nova-3").unwrap(),
            api_key: "test-key".to_string(),
        };
        let (command_tx, command_rx) = mpsc::channel(4);
        let (_abort_tx, abort_rx) = watch::channel(None);
        command_tx
            .send(StreamCommand::Audio(vec![0.0, 1.0, -1.0]))
            .await
            .unwrap();
        command_tx.send(StreamCommand::Finish).await.unwrap();

        let transcript = run(config, command_rx, abort_rx).await.unwrap();
        assert_eq!(transcript.text, "আমি ভালো আছি");
        server.await.unwrap();
    }

    #[tokio::test]
    async fn cancellation_ends_a_pending_connection_without_a_result() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (_socket, _) = listener.accept().await.unwrap();
            tokio::time::sleep(Duration::from_secs(2)).await;
        });
        let config = DeepgramStreamingConfig {
            url: streaming_url(&format!("http://{address}/v1/listen"), "nova-3").unwrap(),
            api_key: "test-key".to_string(),
        };
        let (_command_tx, command_rx) = mpsc::channel(1);
        let (abort_tx, abort_rx) = watch::channel(None);
        let worker = tokio::spawn(run(config, command_rx, abort_rx));
        abort_tx.send(Some(AbortReason::Cancelled)).unwrap();

        let failure = worker.await.unwrap().unwrap_err();
        assert_eq!(failure.error, CloudTranscriptionError::Cancelled);
        assert!(!failure.fallback_allowed);
        server.abort();
    }
}
