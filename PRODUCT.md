# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

People who use a desktop computer and want speech converted into text in the application they are already using. The Bangla workflow serves people who speak Bangla and want the resulting text optionally converted into readable Latin-script Romanization.

## Product Purpose

Kotha is a cross-platform desktop speech-to-text utility. A user presses a global shortcut, speaks, and receives text in the active application. Success means that recording, transcription, optional Romanization, and paste remain fast, legible, and dependable without interrupting the user's current task.

## Positioning

Kotha combines the existing local transcription workflow with a dedicated Bangla route: Deepgram performs Bangla speech recognition and an optional selected LLM provider Romanizes the verified transcript, with raw Bangla used as the defined fallback when Romanization fails.

## Operating Context

Kotha runs primarily from the system tray. Users configure shortcuts, microphones, models, Bangla cloud providers, Romanization, history, and advanced behavior in a compact Tauri settings window. Recording state is communicated through a small overlay while the user's focus remains in another application.

## Capabilities and Constraints

- Preserve the Tauri 2 Rust backend, React and TypeScript frontend, generated bindings, command-event architecture, and Zustand settings flow.
- Preserve all recording, transcription, Bangla, Romanization, cancellation, fallback, privacy, and paste behavior documented in `HANDY_CODEBASE_MAP.md`.
- All user-facing strings use i18next resources.
- Light, dark, RTL, keyboard, and cross-platform behavior must remain supported.
- The first visual rebrand does not migrate the bundle identifier, application data path, updater endpoint, signing configuration, internal shortcut IDs, settings keys, or backend contracts.

## Brand Commitments

- The user-approved product name is Kotha.
- `/Users/rmn/Developer/Romanize/Kotha.jpeg` is the visual authority for the rebrand.
- The recognizable wave, calligraphic K, and green-through-earth-to-vermilion color movement must be preserved when deriving production assets.
- The interface remains a focused desktop utility; branding must not obscure tasks, state, or familiar controls.
- Existing audio feedback sounds remain unchanged.

## Evidence on Hand

- Approved Kotha wordmark: `/Users/rmn/Developer/Romanize/Kotha.jpeg`.
- Implemented product and Bangla architecture: `HANDY_CODEBASE_MAP.md`.
- Existing UI, icon components, app icon set, tray states, overlay, translations, and theme tokens in this repository.
- No original vector artwork is available in the workspace; derived vector and small-size assets must therefore be validated against the supplied JPEG.

## Product Principles

- Preserve the user's flow and focus while speech work happens in the background.
- Make recording and processing state immediately understandable.
- Keep configuration approachable without hiding advanced control.
- Treat privacy-sensitive content and credentials conservatively.
- Express Kotha's identity through precise, consistent details rather than functional disruption.

## Accessibility & Inclusion

Maintain keyboard access, visible focus, readable contrast, light and dark themes, RTL direction, localized copy, and layouts that tolerate longer translations.
