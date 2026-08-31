# Kotha

Kotha is a cross-platform desktop speech-to-text app with a dedicated Bangla transcription and Romanization workflow. Press a global shortcut, speak, and Kotha pastes the result into the application you are using.

Kotha is derived from [Handy](https://github.com/cjpais/Handy) by CJ Pais and its contributors. It keeps Handy's local transcription foundation while adding the Kotha identity and Bangla workflow.

## Features

- Local speech recognition with downloadable Whisper-family and other supported models.
- A dedicated Bangla shortcut using Deepgram transcription.
- Optional Bangla-to-Latin Romanization through Groq, Gemini, or OpenAI.
- Raw Bangla fallback if Romanization fails.
- Configurable global shortcuts and push-to-talk.
- Microphone, channel, output-device, overlay, history, and paste controls.
- Optional transcript post-processing through compatible language-model providers.
- System tray operation on macOS, Windows, and Linux.
- Light and dark Kotha themes.

Local models process audio on the device. The Bangla workflow sends its recording to the configured Deepgram endpoint, and optional Romanization sends the verified Bangla transcript to the selected provider. API credentials are entered by the user.

## Development

Requirements:

- Current stable [Rust](https://rustup.rs/)
- [Bun](https://bun.sh/)
- The platform dependencies in [BUILD.md](BUILD.md)

```bash
git clone https://github.com/RmnFs/kotha.git
cd kotha
bun install
bun run tauri dev
```

Useful checks:

```bash
bun run build
bun run lint
bun run format:check
cargo test --manifest-path src-tauri/Cargo.toml
```

Production bundles are built with:

```bash
bun run tauri build
```

See [BUILD.md](BUILD.md) for platform-specific setup and packaging details.

## Command line

Kotha supports these runtime controls:

```text
kotha --toggle-transcription
kotha --toggle-post-process
kotha --cancel
kotha --start-hidden
kotha --no-tray
kotha --debug
kotha --list-devices
kotha --list-models
```

Run `kotha --help` for the complete list.

## Application data

Kotha uses the application identifier `rmn.kotha`. Standard installations store settings, models, history, and logs under the operating system's application-data location for that identifier:

- macOS: `~/Library/Application Support/rmn.kotha/`
- Windows: `%APPDATA%\rmn.kotha\`
- Linux: `~/.config/rmn.kotha/`

Portable Windows installations store their data in a `Data` directory beside the executable.

The downloadable model catalog is compiled into Kotha. Model files are currently fetched from the Handy project's public model infrastructure and Hugging Face repositories; those external identifiers intentionally remain unchanged.

## Troubleshooting

### macOS Secure Input

If shortcuts stop responding on macOS, another application may have enabled Secure Input. Kotha displays a warning when it can identify this state. Quit password fields, terminal password prompts, or other applications that may be holding Secure Input, then try the shortcut again.

### Linux overlay startup

The recording overlay uses GTK Layer Shell. To diagnose compositor compatibility, start Kotha with:

```bash
KOTHA_NO_GTK_LAYER_SHELL=1 kotha
```

On Wayland, pasting may require `wtype` or `ydotool`, depending on the compositor.

### Logs

Open **Settings → About → Log Directory**. Remove transcripts, API keys, endpoints, and other private content before sharing logs.

## Releases and updates

GitHub Actions can build macOS, Windows, and Linux installers through the manually triggered release workflow. Automatic in-app updates are currently disabled until Kotha has its own updater signing key. Release artifacts may also show operating-system warnings until platform code-signing certificates are configured.

## Issues

Reproducible bugs can be reported through [GitHub Issues](https://github.com/RmnFs/kotha/issues). This repository is not currently seeking external pull requests.

## License and lineage

Kotha is distributed under the [MIT License](LICENSE). The original Handy copyright and license notice are preserved as required.

Upstream project: [cjpais/Handy](https://github.com/cjpais/Handy)

Kotha also depends on open-source projects including Tauri, React, transcribe.cpp, transcribe-rs, ggml, and their respective dependencies.
