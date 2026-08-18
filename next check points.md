# Next Checkpoints — Bangla Transcription and Romanization

This roadmap focuses on understanding and reducing the time between releasing the Bangla hotkey and receiving the final pasted result. It must preserve transcription quality, Romanization quality, privacy, cancellation, and raw-Bangla fallback behavior.

The supported model choices are already settled for this work:

- Deepgram transcription remains `nova-3` with Bengali (`bn`).
- Groq Romanization remains `openai/gpt-oss-120b`. The 20B model has already been tested and is not accurate enough for this use case.
- Gemini Romanization remains `gemini-3.1-flash-lite`. Gemini 2.5 Flash-Lite is unavailable to the recent API key being used and will not be evaluated.
- This roadmap does not include testing alternative Groq or Gemini models.

## Checkpoint 1 — Clear, privacy-safe diagnostics inside the app

### Goal

Make Bangla performance easy to understand without reading log files. When Debug Mode is enabled, the Bangla settings screen should show how long transcription, Romanization, and the complete operation took for the most recent Bangla request.

### User interface

Add a **Last Bangla diagnostic** card to the Bangla settings screen. Keep the summary visible and make the complete breakdown expandable.

The default summary should be immediately readable:

```text
Total                 1.84 s
Transcription         0.52 s   Deepgram · nova-3 · Streaming
Romanization          1.27 s   Groq · openai/gpt-oss-120b
Result                Romanized and pasted
```

The expanded details should show only content-free operational information:

- Recording duration.
- Recorder stop and final-frame drain time.
- Transcription provider and model.
- Batch, streaming, or streaming-to-batch-fallback transport.
- Deepgram finalization and total transcription time.
- Romanization provider and model.
- Romanization request-to-response-headers time.
- Response download and parsing time.
- Provider-supplied server timing when safely available.
- Main-thread paste queue and paste-call time.
- Total post-release time and total recording-to-terminal time.
- Stable success, cancellation, fallback, or error category.
- Provider request ID when one is safely available.

The card should support two views:

1. **Summary:** transcription, Romanization, total time, selected models, and final outcome.
2. **Details:** the complete stage-by-stage breakdown for diagnosis.

Keep only the latest diagnostic snapshot in memory for the current app session. Do not write diagnostic history to disk. Include a clear action that removes the displayed snapshot.

### Backend diagnostics

1. Replace or supplement the current text-only `bangla_latency` log with a typed, content-free diagnostic payload that can be emitted to the frontend.
2. Record the same diagnostic payload for successful, failed, cancelled, and batch-fallback operations.
3. Parse provider-supplied performance metadata when available:
   - Groq queue, prompt-processing, completion, and total server time.
   - Gemini prompt, candidate, and thinking-token counts.
4. Continue producing a compact safe log line for terminal/log-based debugging.
5. Never include audio, transcript text, Romanized text, prompt content, API keys, authorization headers, configured endpoints, or raw provider responses.

### Likely code areas

- `src-tauri/src/actions.rs`
- `src-tauri/src/bangla_romanization.rs`
- `src-tauri/src/bangla_transcription/`
- `src-tauri/src/lib.rs`
- `src/components/settings/bangla/BanglaSettings.tsx`
- A small Bangla diagnostic component under `src/components/settings/bangla/`
- `src/stores/settingsStore.ts` or a session-only diagnostic store
- `src/bindings.ts`, regenerated after adding the typed event payload
- `src/i18n/locales/*/translation.json`
- `HANDY_CODEBASE_MAP.md`

### Acceptance criteria

- With Debug Mode enabled, the Bangla settings screen shows the latest operation in both summary and detailed forms.
- Transcription and Romanization times can be understood at a glance.
- The displayed provider, model, and STT transport match the settings actually used for that operation.
- Streaming-to-batch fallback is clearly identified.
- A failed or cancelled operation still produces a useful content-free diagnostic.
- No diagnostic value contains user content or credentials.
- Diagnostics are session-only and are not written to settings, history, or another file.
- With Debug Mode disabled, no diagnostic snapshot is retained or displayed.

## Checkpoint 2 — Improvements using the existing supported models

### Goal

Reduce latency without changing the supported Groq or Gemini models and without weakening reliability.

### Recommended implementation order

1. **Reuse HTTP clients and connections.**

   Maintain long-lived `reqwest::Client` instances instead of constructing a new client for every Romanization request. Send authentication per request so changing an API key cannot leave stale credentials in a pooled client's default headers. Preserve the current timeout and cancellation behavior.

2. **Use explicit low-latency reasoning controls for the supported models.**

   Romanization is a constrained text transformation and does not need deep reasoning.
   - Groq `openai/gpt-oss-120b`: request `reasoning_effort: "low"`.
   - Gemini `gemini-3.1-flash-lite`: explicitly request its minimal supported thinking level.
   - Do not send these options to unrelated or user-entered model IDs unless their compatibility is known.

3. **Add a conservative output limit.**

   Calculate an output-token limit from the input size with generous minimum and maximum bounds. Allow room for Bangla-to-Latin expansion and the JSON wrapper. If a response is truncated or malformed, keep the existing safe behavior: show an error and paste the verified Bangla transcript.

4. **Measure every optimization independently.**

   Use the diagnostics from Checkpoint 1 to compare before and after results. Measure cold and warm requests separately and compare median and p95 latency. Do not combine several optimizations into one measurement because that would hide which change helped or caused a regression.

5. **Evaluate prompt changes only after the preceding work.**

   Keep the current safety, non-translation, meaning-preservation, and structured-output requirements. Only shorten or reorganize the prompt if an A/B test with the existing models demonstrates a meaningful improvement without reducing Romanization quality or schema reliability.

### Likely code areas

- `src-tauri/src/bangla_romanization.rs`
- `src-tauri/src/lib.rs` for shared HTTP client state
- `src-tauri/src/actions.rs`
- Tests beside `bangla_romanization.rs`
- `HANDY_CODEBASE_MAP.md`

### Acceptance criteria

- Warm Romanization requests demonstrate reduced connection overhead.
- Groq 120B and Gemini 3.1 Flash-Lite receive only parameters they support.
- Custom or unknown model IDs retain a compatible request path.
- The output limit does not truncate the longest supported test case.
- Romanization still runs exactly once after the complete verified transcript.
- Cancellation still suppresses late results and prevents a late paste.
- Romanization failure still emits a safe error and pastes verified Bangla text.
- No optimization increases malformed responses or reduces reviewed Romanization quality.

## Checkpoint 3 — Best course of action

### Recommended sequence

1. Implement the in-app summary and detailed diagnostic views.
2. Verify the diagnostic payload and UI against successful batch, successful streaming, streaming fallback, Romanization failure, cancellation, and paste failure.
3. Record a baseline using only the supported configurations:
   - Deepgram `nova-3` batch and streaming.
   - Groq `openai/gpt-oss-120b`.
   - Gemini `gemini-3.1-flash-lite`.
4. Collect enough repeated requests to calculate median and p95 latency rather than relying on a single fast or slow result.
5. Implement reusable HTTP clients and compare the results with the baseline.
6. Add explicit low-latency reasoning controls and compare again.
7. Add and test the conservative output limit.
8. Consider a prompt revision only if the remaining delay is demonstrably related to prompt or model processing rather than network or provider queueing.
9. Keep each optimization in a separate commit so it can be reviewed, measured, and reverted independently.
10. Update `HANDY_CODEBASE_MAP.md` and tests with the final diagnostic and connection-lifecycle contracts.

### Decision criteria

Keep an optimization only when it:

- Preserves the meaning and naturalness of Romanization.
- Does not increase malformed or empty responses.
- Produces a measurable improvement in median or p95 post-release latency.
- Preserves privacy, cancellation, and raw-Bangla fallback guarantees.
- Works in both Deepgram batch and streaming modes.
- Passes the full automated test suite and manual hotkey testing.

### Product contracts that must remain unchanged

- Batch and streaming Deepgram modes remain selectable.
- Streaming may use one recoverable batch fallback.
- Romanization happens once after the complete transcript.
- Cancellation prevents late network results from being pasted.
- Romanization failure pastes the verified Bangla transcript.
- Debug Mode never records or displays audio, transcript text, Romanized text, prompts, API keys, endpoints, authorization headers, or raw provider responses.

## Technical references

- [Groq latency optimization](https://console.groq.com/docs/production-readiness/optimizing-latency)
- [Groq API reasoning controls](https://console.groq.com/docs/api-reference)
- [Groq GPT-OSS 120B](https://console.groq.com/docs/model/openai/gpt-oss-120b)
- [Gemini thinking controls](https://ai.google.dev/gemini-api/docs/generate-content/thinking)
- [Gemini 3.1 Flash-Lite](https://ai.google.dev/gemini-api/docs/models/gemini-3.1-flash-lite)
- [Gemini structured output](https://ai.google.dev/gemini-api/docs/generate-content/structured-output)
