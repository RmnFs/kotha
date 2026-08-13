//! Deepgram Nova-3 batch adapter.
//!
//! Contract (verified 2026-08): `POST /v1/listen` with `Authorization: Token
//! <API key>`, binary `audio/wav`, `model=nova-3`, `language=bn`, and a result
//! at `results.channels[0].alternatives[0].transcript`. Keep Deepgram-specific
//! transport details here so a provider swap does not affect actions or audio.

use super::{
    BanglaTranscript, BanglaTranscriptionProvider, CancellationContext, CloudTranscriptionError,
    RecordedAudio, TranscriptionFuture,
};
use crate::settings::AppSettings;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest::{StatusCode, Url};
use serde::Deserialize;
use std::time::Duration;

pub(crate) const DEEPGRAM_PROVIDER_ID: &str = "deepgram";
const DEFAULT_DEEPGRAM_ENDPOINT: &str = "https://api.deepgram.com/v1/listen";
const DEFAULT_DEEPGRAM_MODEL: &str = "nova-3";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(45);

pub(crate) struct DeepgramNova3Provider;

impl BanglaTranscriptionProvider for DeepgramNova3Provider {
    fn transcribe<'a>(
        &'a self,
        audio: RecordedAudio,
        settings: &'a AppSettings,
        cancellation: CancellationContext,
    ) -> TranscriptionFuture<'a> {
        Box::pin(async move {
            if cancellation.is_cancelled() {
                return Err(CloudTranscriptionError::Cancelled);
            }

            let api_key = settings
                .bangla_stt_api_keys
                .get(DEEPGRAM_PROVIDER_ID)
                .map(String::as_str)
                .unwrap_or("")
                .trim();
            if api_key.is_empty() {
                return Err(CloudTranscriptionError::MissingConfiguration);
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
                return Err(CloudTranscriptionError::MissingConfiguration);
            }

            let url = deepgram_url(endpoint, model)?;
            let wav = encode_wav(&audio)?;
            let client = deepgram_client(api_key)?;

            let response = client
                .post(url)
                .body(wav)
                .send()
                .await
                .map_err(map_request_error)?;
            if cancellation.is_cancelled() {
                return Err(CloudTranscriptionError::Cancelled);
            }

            if !response.status().is_success() {
                return Err(map_status(response.status()));
            }

            let body = response
                .json::<DeepgramResponse>()
                .await
                .map_err(|_| CloudTranscriptionError::MalformedResponse)?;
            if cancellation.is_cancelled() {
                return Err(CloudTranscriptionError::Cancelled);
            }

            extract_transcript(body)
        })
    }
}

fn deepgram_url(endpoint: &str, model: &str) -> Result<Url, CloudTranscriptionError> {
    let mut url =
        Url::parse(endpoint).map_err(|_| CloudTranscriptionError::InvalidConfiguration)?;
    if !matches!(url.scheme(), "https" | "http") {
        return Err(CloudTranscriptionError::InvalidConfiguration);
    }
    {
        let mut query = url.query_pairs_mut();
        query.append_pair("model", model);
        query.append_pair("language", "bn");
        query.append_pair("smart_format", "true");
    }
    Ok(url)
}

fn deepgram_client(api_key: &str) -> Result<reqwest::Client, CloudTranscriptionError> {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("audio/wav"));
    let auth = HeaderValue::from_str(&format!("Token {api_key}"))
        .map_err(|_| CloudTranscriptionError::InvalidConfiguration)?;
    headers.insert(AUTHORIZATION, auth);
    reqwest::Client::builder()
        .default_headers(headers)
        .connect_timeout(REQUEST_TIMEOUT)
        .timeout(REQUEST_TIMEOUT)
        .build()
        .map_err(|_| CloudTranscriptionError::Provider)
}

fn encode_wav(audio: &RecordedAudio) -> Result<Vec<u8>, CloudTranscriptionError> {
    if audio.sample_rate != 16_000 || audio.channels != 1 {
        return Err(CloudTranscriptionError::InvalidConfiguration);
    }

    // A minimal canonical PCM WAV header avoids a temporary file and keeps the
    // provider-specific payload conversion self-contained. The recorder data
    // is always mono 16 kHz, and samples are clamped before i16 conversion.
    let data_len = audio
        .samples
        .len()
        .checked_mul(2)
        .ok_or(CloudTranscriptionError::AudioTooLong)?;
    let riff_len = 36usize
        .checked_add(data_len)
        .ok_or(CloudTranscriptionError::AudioTooLong)?;
    let data_len_u32 =
        u32::try_from(data_len).map_err(|_| CloudTranscriptionError::AudioTooLong)?;
    let riff_len_u32 =
        u32::try_from(riff_len).map_err(|_| CloudTranscriptionError::AudioTooLong)?;
    let byte_rate = audio.sample_rate * u32::from(audio.channels) * 2;
    let block_align = audio.channels * 2;
    let mut wav = Vec::with_capacity(44 + data_len);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&riff_len_u32.to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&audio.channels.to_le_bytes());
    wav.extend_from_slice(&audio.sample_rate.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_len_u32.to_le_bytes());
    for sample in &audio.samples {
        let pcm = (sample.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16;
        wav.extend_from_slice(&pcm.to_le_bytes());
    }
    Ok(wav)
}

fn map_request_error(error: reqwest::Error) -> CloudTranscriptionError {
    if error.is_timeout() {
        CloudTranscriptionError::Timeout
    } else if error.is_connect() {
        CloudTranscriptionError::Offline
    } else {
        CloudTranscriptionError::Provider
    }
}

fn map_status(status: StatusCode) -> CloudTranscriptionError {
    match status {
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => CloudTranscriptionError::Authentication,
        StatusCode::TOO_MANY_REQUESTS => CloudTranscriptionError::RateLimited,
        status if status.is_server_error() => CloudTranscriptionError::Provider,
        _ => CloudTranscriptionError::Provider,
    }
}

fn extract_transcript(
    response: DeepgramResponse,
) -> Result<BanglaTranscript, CloudTranscriptionError> {
    let transcript = response
        .results
        .channels
        .first()
        .and_then(|channel| channel.alternatives.first())
        .map(|alternative| alternative.transcript.clone())
        .ok_or(CloudTranscriptionError::MalformedResponse)?;
    BanglaTranscript::new(transcript)
}

#[derive(Deserialize)]
struct DeepgramResponse {
    results: DeepgramResults,
}

#[derive(Deserialize)]
struct DeepgramResults {
    channels: Vec<DeepgramChannel>,
}

#[derive(Deserialize)]
struct DeepgramChannel {
    alternatives: Vec<DeepgramAlternative>,
}

#[derive(Deserialize)]
struct DeepgramAlternative {
    transcript: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::get_default_settings;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    #[test]
    fn creates_a_16khz_mono_pcm_wav() {
        let bytes = encode_wav(&RecordedAudio {
            samples: vec![0.0, 1.0, -1.0],
            sample_rate: 16_000,
            channels: 1,
        })
        .unwrap();
        assert_eq!(&bytes[..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE");
        assert_eq!(bytes.len(), 44 + 6);
    }

    #[test]
    fn deepgram_url_includes_batch_bangla_parameters() {
        let url = deepgram_url("https://api.deepgram.com/v1/listen", "nova-3").unwrap();
        let query: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(query["model"], "nova-3");
        assert_eq!(query["language"], "bn");
        assert_eq!(query["smart_format"], "true");
    }

    #[test]
    fn maps_actionable_http_statuses() {
        assert_eq!(
            map_status(StatusCode::UNAUTHORIZED),
            CloudTranscriptionError::Authentication
        );
        assert_eq!(
            map_status(StatusCode::TOO_MANY_REQUESTS),
            CloudTranscriptionError::RateLimited
        );
        assert_eq!(
            map_status(StatusCode::BAD_GATEWAY),
            CloudTranscriptionError::Provider
        );
    }

    #[test]
    fn extracts_only_the_first_final_channel_alternative() {
        let response: DeepgramResponse = serde_json::from_value(serde_json::json!({
            "results": { "channels": [{ "alternatives": [{ "transcript": "আমি ভালো আছি" }] }] }
        }))
        .unwrap();
        assert_eq!(extract_transcript(response).unwrap().text, "আমি ভালো আছি");
    }

    #[test]
    fn rejects_missing_or_empty_provider_transcripts() {
        let missing: DeepgramResponse = serde_json::from_value(serde_json::json!({
            "results": { "channels": [] }
        }))
        .unwrap();
        assert_eq!(
            extract_transcript(missing).unwrap_err(),
            CloudTranscriptionError::MalformedResponse
        );
        let empty: DeepgramResponse = serde_json::from_value(serde_json::json!({
            "results": { "channels": [{ "alternatives": [{ "transcript": " " }] }] }
        }))
        .unwrap();
        assert_eq!(
            extract_transcript(empty).unwrap_err(),
            CloudTranscriptionError::EmptyTranscript
        );
    }

    #[tokio::test]
    async fn adapter_posts_wav_with_token_auth_and_parses_the_batch_response() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = vec![0; 2048];
            let read = socket.read(&mut request).await.unwrap();
            let request = String::from_utf8_lossy(&request[..read]);
            assert!(request.starts_with("POST /v1/listen?"));
            assert!(request.contains("authorization: Token test-key"));
            assert!(request.contains("content-type: audio/wav"));
            assert!(request.contains("RIFF"));

            let body =
                r#"{"results":{"channels":[{"alternatives":[{"transcript":"আমি ভালো আছি"}]}]}}"#;
            socket
                .write_all(
                    format!(
                        "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                        body.len(), body
                    )
                    .as_bytes(),
                )
                .await
                .unwrap();
        });

        let mut settings = get_default_settings();
        settings.bangla_stt_endpoint = format!("http://{address}/v1/listen");
        settings
            .bangla_stt_api_keys
            .insert(DEEPGRAM_PROVIDER_ID.to_string(), "test-key".to_string());
        let transcript = DeepgramNova3Provider
            .transcribe(
                RecordedAudio {
                    samples: vec![0.0; 160],
                    sample_rate: 16_000,
                    channels: 1,
                },
                &settings,
                CancellationContext::new(|| false),
            )
            .await
            .unwrap();
        assert_eq!(transcript.text, "আমি ভালো আছি");
        server.await.unwrap();
    }
}
