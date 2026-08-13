

## Checkpoint 1 — Second hotkey and Bangla mode skeleton

Goal: establish a distinct Bangla activation path without making network requests.

### Implementation scope

Introduce a stable mode concept, such as:

```rust
enum TranscriptionMode {
    Local,
    LocalWithPostProcessing,
    BanglaRomanization,
}
```

The binding ID selects the mode; no global mode selector is necessary.

Add a new binding ID, for example:

```text
transcribe_bangla_romanized
```

Update:

- `src-tauri/src/settings.rs`
  - Add the default binding.
  - Ensure existing settings stores receive the missing binding automatically.
  - Add settings migration/default tests.

- `src-tauri/src/transcription_coordinator.rs`
  - Recognize the Bangla binding.
  - Preserve its binding/mode identity through recording and processing.
  - Maintain the one-operation-at-a-time invariant.

- `src-tauri/src/actions.rs`
  - Add the Bangla action to `ACTION_MAP`.
  - Start the shared recording flow without loading a local model.
  - Stop recording and safely discard the audio for this checkpoint.
  - Do not transcribe or paste anything yet.

- `src-tauri/src/shortcut/tauri_impl.rs`
- `src-tauri/src/shortcut/handy_keys.rs`
- `src-tauri/src/shortcut/handler.rs`
  - Register and route the new binding through both shortcut implementations.

- `src-tauri/src/secure_input.rs`
  - Include the new binding in the macOS Carbon fallback behavior.

- `src/components/settings/ShortcutInput.tsx`
- `src/components/settings/general/GeneralSettings.tsx`, or a new Bangla settings section
- `src/i18n/locales/*/translation.json`
  - Show and configure the new shortcut.

Use the existing audio manager with `VadPolicy::Offline` when VAD is enabled, otherwise `Disabled`. Do not start the local `StreamRouter`.

### Verification checkpoint

You should be able to verify:

- The Bangla shortcut appears in settings.
- It can be changed, reset, and survives an application restart.
- Pressing it starts the normal recording overlay, tray state, audio feedback, and microphone capture.
- Releasing/toggling it stops recording cleanly.
- Escape cancellation works.
- It does not load a local model.
- It does not transcribe or paste anything.
- Existing local and post-processed English hotkeys behave exactly as before.
- Shortcut conflicts are rejected correctly.
- On macOS, Secure Input diagnostics account for the new binding.

### Tests

Add tests for:

- Missing binding migration into existing settings.
- Coordinator recognition of the new binding.
- Cross-mode input while recording/processing.
- Push-to-talk and toggle behavior.
- Cancellation and finish-state cleanup.

This checkpoint establishes the architectural branch and is the most important foundation.

---

## Checkpoint 2 — Batch cloud Bangla transcription

Goal: make the Bangla hotkey produce verified Bangla-script transcription, without Romanization yet.

Before starting this checkpoint, choose the cloud STT service and obtain its exact:

- Authentication method
- Accepted audio format
- Request endpoint/schema
- Response schema
- Maximum duration/size
- Timeout and rate-limit behavior

The application architecture should still expose a provider-neutral contract even though the first implementation uses one provider.

### Service contract

Add a backend service boundary resembling:

```rust
struct RecordedAudio {
    samples: Vec<f32>,
    sample_rate: u32, // 16_000
    channels: u16,    // 1
}

struct BanglaTranscript {
    text: String,
}

async fn transcribe_bangla(
    audio: RecordedAudio,
    cancellation: CancellationContext,
) -> Result<BanglaTranscript, CloudTranscriptionError>;
```

The service adapter is responsible for converting `Vec<f32>` into the provider’s required WAV/PCM/other payload. Do not perform network work from the real-time audio callback.

### Integration scope

Update:

- `src-tauri/src/actions.rs`
  - After the Bangla recording stops, route samples to the cloud service instead of `TranscriptionManager`.
  - Bypass local model loading, `finalize_stream`, local normalization, language detection, and local batch inference.
  - Show the existing Transcribing state during the request.
  - Suppress late results after cancellation.

- `src-tauri/src/lib.rs::initialize_core_logic`
  - Register the cloud service as managed state if appropriate.

- `src-tauri/src/settings.rs`
- `src/stores/settingsStore.ts`
- Bangla settings UI
  - Add the minimum endpoint/model/auth configuration required by the selected provider.
  - Do not reuse `selected_model`.
  - Do not embed an application-owned secret in the binary.

- Add a bounded request timeout.
- Validate successful responses and reject missing/empty text.
- Map offline, authentication, rate-limit, timeout, provider, and malformed-response errors separately where possible.
- Redact API keys, audio, transcription content, and sensitive URLs from logs.

### Checkpoint-only output behavior

For this intermediate checkpoint, raw Bangla may be pasted through the existing insertion function solely to verify STT:

```text
Bangla hotkey
→ recording
→ cloud STT
→ raw Bangla pasted
```

This is explicitly temporary checkpoint behavior. It must be replaced in Checkpoint 3 and must not remain as the failure fallback for Romanization.

Alternatively, raw Bangla can be exposed through a development-only result view if you prefer never to paste intermediate text.

### Verification checkpoint

Test with known Bangla phrases and confirm:

- Only the Bangla shortcut makes a network request.
- The cloud receives valid audio.
- Returned Bangla is accurate enough for Romanization.
- Raw Bangla appears through the chosen checkpoint inspection method.
- Local English remains offline and unchanged.
- Cancelling during upload/request produces no paste.
- Timeout, invalid key, offline state, empty response, and server error all return the app to Idle.
- A failed cloud call never falls back to the local English model.
- Repeated activation cannot create overlapping recordings or requests.

Do not add streaming yet.

---

## Checkpoint 3 — Required LLM Romanization and final insertion

Goal: complete the intended feature.

The pipeline becomes:

```text
Bangla hotkey
→ recording
→ cloud Bangla STT
→ Bangla text
→ required LLM Romanization
→ Romanized text
→ existing paste layer
```

### Romanization contract

Create a semantic boundary separate from optional English polishing:

```rust
struct RomanizationInput {
    bangla_text: String,
}

struct RomanizationResult {
    romanized_text: String,
}

async fn romanize_bangla(
    input: RomanizationInput,
    cancellation: CancellationContext,
) -> Result<RomanizationResult, RomanizationError>;
```

The existing `src-tauri/src/llm_client.rs` may provide reusable HTTP mechanics, but do not reuse the fail-open semantics of `post_process_transcription`.

### Required behavior

- After cloud STT succeeds, change the overlay from Transcribing to Processing/Romanizing.
- Send Bangla text to the configured Romanization LLM.
- Validate that the returned result is non-empty.
- Paste only the successful Romanized result using:
  - `src-tauri/src/clipboard.rs::paste`
  - Existing paste method, clipboard restoration, trailing-space, and auto-submit settings.

Romanization failure must behave as:

```text
STT succeeds
Romanization fails
→ show an error
→ paste nothing
```

It must not silently paste raw Bangla.

### Settings

Keep Romanization configuration separate from optional local-transcription post-processing:

- Romanization enabled implicitly by the Bangla mode
- Romanization provider/model
- Credential reference
- Prompt/version if user-configurable
- Timeout

A fixed internal prompt is acceptable initially, but it should be isolated and testable rather than embedded throughout `actions.rs`.

### Verification checkpoint

Use a small test set covering:

- Normal Bangla conversation
- Proper names
- English words within Bangla sentences
- Numbers and punctuation
- Short utterances
- Empty/silent audio
- Mixed Bangla-English speech

Verify:

- Bangla STT output is sent to Romanization.
- Only Romanized text is pasted.
- Existing English post-processing settings do not affect Bangla Romanization.
- Romanization failure pastes nothing.
- STT failure never calls the LLM.
- Cancelling during either network stage produces no late paste.
- Overlay/tray always return to Idle.
- Local English continues working without cloud credentials.

At this point, the core feature is functionally complete.

---

## Checkpoint 4 — Durability, history, privacy, and platform hardening

Goal: turn the functional pipeline into a dependable daily-use feature.

### History and retry

Migrate `src-tauri/src/managers/history.rs` to distinguish modes and stages. At minimum, retain:

- Transcription mode
- Saved audio path
- Raw Bangla text
- Romanized result
- Completion/failure stage
- Whether Romanization was requested

Update `src-tauri/src/commands/history.rs::retry_history_entry_transcription`:

- Existing local entries continue using local inference.
- Bangla entries use the cloud pipeline.
- If Bangla STT already succeeded but Romanization failed, allow Romanization-only retry without retranscribing the audio.
- Old rows must continue deserializing as local-mode history.

Update `src/components/settings/history/` to label local and Bangla entries clearly.

### Reliability

Implement and verify:

- Connect and total-request timeouts
- Actual HTTP request cancellation where supported
- Late-result suppression through the existing generation/coordinator checks
- Bounded retry only for safe transient failures
- Explicit handling for HTTP 401/403, 429, 5xx, malformed responses, and offline state
- Maximum recording duration/payload validation
- No retry after user cancellation
- No duplicate paste after retry

### Privacy and credentials

- Never bundle a reusable provider key.
- At minimum, redact credentials and content from logs and UI errors.
- Decide whether JSON key storage is acceptable for the private build.
- Before wider distribution, use macOS Keychain and Windows Credential Manager or another deliberate secure-storage mechanism.
- Update `src-tauri/Info.plist`; its current microphone description says audio is transcribed locally.
- Add a clear cloud-audio disclosure in Bangla settings/onboarding.
- Document whether the STT and LLM providers retain audio/text.

### Focus safety

Handy does not record or restore the original focused field. Test the following explicitly:

1. Start Bangla recording in application A.
2. Switch to application B while cloud processing is running.
3. Observe where the result is pasted.

Then make a product decision:

- Accept “paste into whichever field is active when processing finishes,” or
- Add a confirmation/ready-to-paste flow, or
- Later build platform-specific focus-target validation.

The first option is simplest for a private tool but should be understood.

### Final regression matrix

macOS first:

- Accessibility permission granted/denied
- Microphone permission granted/denied
- Secure Event Input
- Push-to-talk and toggle
- Escape cancellation
- Non-activating overlay
- Clipboard and direct typing
- Focus changes during cloud processing
- Local English batch and streaming
- Bangla success and every failure stage

Windows:

- x64 and ARM64 startup
- Microphone privacy denial
- HandyKeys and Tauri shortcut implementations
- Delayed-render clipboard paste
- Network timeout/cancellation
- Local English regression
- Installer/package contents

Run:

- Rust tests
- Frontend type-check/build
- ESLint
- Translation validation
- Prettier/Rust formatting
- Relevant platform packaging builds

## Explicit non-goals for these checkpoints

Defer all of the following:

- Automatic English/Bangla language detection
- Cloud English transcription
- Cloud streaming/partial results
- Removing local models
- Replacing the existing English post-processing path
- Multiple interchangeable cloud STT providers
- Account systems or an application-owned API proxy
- Explicit focused-field discovery/highlighting

## Final definition of done

The feature is complete when:

```text
English hotkey
→ existing local transcription
→ existing optional polishing
→ existing insertion

Bangla hotkey
→ shared recording
→ cloud Bangla transcription
→ required Romanization
→ shared insertion
```

Both modes must have independent configurable hotkeys, share the one-operation coordinator safely, persist across restarts, cancel without late insertion, report stage-specific failures, retain correct history, and avoid changing the current local English behavior.