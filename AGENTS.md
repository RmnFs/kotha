# Kotha agent guide

This file gives coding agents the repository-specific context needed to change Kotha safely.

## Commands

```bash
bun install
bun run tauri dev
bun run build
bun run lint
bun run format:check
cargo test --manifest-path src-tauri/Cargo.toml
```

Run `bun run tauri build` only when a platform bundle is required. Platform prerequisites are documented in `BUILD.md`.

## Architecture

Kotha is a Tauri 2 desktop application:

- `src/`: React and TypeScript settings UI and recording overlay.
- `src-tauri/src/`: Rust application, audio, transcription, Bangla, shortcuts, paste, tray, and storage logic.
- `src-tauri/src/catalog/catalog.json`: model catalog compiled into the app.
- `src/bindings.ts`: generated Tauri-Specta commands, events, and types.
- `KOTHA_CODEBASE_MAP.md`: detailed Bangla transcription and Romanization contracts.

The main pipeline is audio → voice activity detection → transcription → optional post-processing or Romanization → clipboard/paste. Preserve cancellation, raw-Bangla fallback, focus behavior, privacy boundaries, and single-instance CLI routing.

## Product contracts

- Keep macOS, Windows, and Linux support.
- The interface is English-only. User-facing React text still belongs in `src/i18n/locales/en/translation.json` and is accessed through i18next.
- Interface language and speech-recognition language are separate concerns. Do not remove model-language or Bangla controls.
- Standard transcription remains local. Bangla cloud transcription and optional Romanization must remain explicit in settings.
- Never log audio, transcript text, Romanized text, prompts, API keys, authorization headers, or raw provider responses in diagnostics.
- Preserve raw Bangla paste fallback when Romanization fails.
- Preserve the `rmn.kotha` application identifier unless a deliberate data migration is part of the change.

## External Handy identifiers

Kotha is derived from Handy and still consumes infrastructure and libraries whose proper names contain `handy`. Do not rename or remove these merely to eliminate the word:

- `handy-keys` dependency, implementation names, and event contracts.
- `handy-computer/*` Hugging Face repositories.
- `blob.handy.computer` model and runtime downloads.
- Required `cjpais` dependency forks pinned in `Cargo.toml` and `Cargo.lock`.

Product-owned names, paths, logs, executable names, package metadata, and release artifacts should use Kotha.

## Generated bindings

Do not hand-edit `src/bindings.ts`. Debug application startup exports it from the command and event registry in `src-tauri/src/lib.rs`. Regenerate it after changing Specta commands or settings types, then verify the frontend build.

## Code style

- Rust: run `cargo fmt`; handle expected errors instead of adding production `unwrap` calls.
- TypeScript: keep strict types, functional components, and existing semantic theme tokens.
- Preserve user changes in a dirty worktree and avoid unrelated rewrites.
- Add or update focused tests when changing routing, settings migration, cancellation, downloads, or text transformation.

## Debugging

Debug settings are opened with `Cmd+Shift+D` on macOS or `Ctrl+Shift+D` on Windows/Linux. Useful Kotha-only diagnostic environment variables include:

- `KOTHA_NO_GTK_LAYER_SHELL=1`
- `KOTHA_FORCE_AI_STUB=1`
- `KOTHA_FORCE_TRANSCRIPTION_FAILURE=1`

Model-host identifiers and `handy-keys` names are not Kotha branding residue; see the external-identifier section above.
