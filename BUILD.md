# Building Kotha

Kotha is a Tauri 2 desktop application with a React/TypeScript frontend and Rust backend.

## Common prerequisites

- Current stable [Rust](https://rustup.rs/)
- [Bun](https://bun.sh/)
- [Tauri 2 prerequisites](https://tauri.app/start/prerequisites/)
- CMake and a C/C++ toolchain for the native transcription libraries

Clone and install:

```bash
git clone https://github.com/RmnFs/kotha.git
cd kotha
bun install
```

Run the application:

```bash
bun run tauri dev
```

Build the frontend or a native bundle:

```bash
bun run build
bun run tauri build
```

## macOS

Install Xcode or the Xcode Command Line Tools:

```bash
xcode-select --install
```

Apple Silicon uses Metal acceleration. Intel macOS builds need a dynamically linked ONNX Runtime:

```bash
brew install onnxruntime
ORT_LIB_LOCATION="$(brew --prefix onnxruntime)/lib" ORT_PREFER_DYNAMIC_LINK=1 bun run tauri dev
```

Use the same environment variables with `bun run tauri build` for an Intel production bundle.

The Apple Intelligence bridge requires a compatible full Xcode installation. To compile with the stub when that SDK is unavailable:

```bash
KOTHA_FORCE_AI_STUB=1 bun run tauri dev
```

## Windows

Install:

- Visual Studio 2022 Build Tools with **Desktop development with C++**
- CMake
- LunarG Vulkan SDK

```powershell
winget install Kitware.CMake
winget install KhronosGroup.VulkanSDK
```

Open a new terminal after installing the Vulkan SDK. `transcribe-cpp` mitigates the traditional 260-character build-path limit automatically; if corporate policy prevents its short-path junction, use a short Cargo target directory:

```powershell
$env:CARGO_TARGET_DIR = "C:\kotha-target"
bun run tauri build
```

Kotha does not currently configure Windows code signing. Locally built installers and public release artifacts may trigger SmartScreen warnings.

## Linux

Ubuntu/Debian dependencies:

```bash
sudo apt update
sudo apt install build-essential clang libclang-dev libevdev-dev libasound2-dev \
  pkg-config libssl-dev libvulkan-dev vulkan-tools glslc spirv-headers \
  glslang-tools libgtk-3-dev libwebkit2gtk-4.1-dev \
  libayatana-appindicator3-dev librsvg2-dev libgtk-layer-shell0 \
  libgtk-layer-shell-dev libopenblas-dev patchelf cmake
```

Fedora/RHEL dependencies:

```bash
sudo dnf groupinstall "Development Tools"
sudo dnf install alsa-lib-devel pkgconf openssl-devel vulkan-devel glslc \
  clang clang-devel libevdev-devel spirv-headers-devel spirv-tools-devel \
  glslang gtk3-devel webkit2gtk4.1-devel libappindicator-gtk3-devel \
  librsvg2-devel gtk-layer-shell gtk-layer-shell-devel openblas-devel cmake
```

Arch dependencies:

```bash
sudo pacman -S base-devel clang libevdev shaderc spirv-headers glslang \
  alsa-lib pkgconf openssl vulkan-devel gtk3 webkit2gtk-4.1 \
  libappindicator-gtk3 librsvg gtk-layer-shell openblas cmake
```

The `.deb` and `.rpm` bundles install native runtime libraries under `/usr/lib/Kotha/`; the executable's rpath resolves that private directory. AppImage keeps its libraries inside the image.

To diagnose GTK Layer Shell compatibility:

```bash
KOTHA_NO_GTK_LAYER_SHELL=1 bun run tauri dev
```

## Tests and formatting

```bash
bun run build
bun run lint
bun run format:check
cargo test --manifest-path src-tauri/Cargo.toml
```

## GitHub builds

- `checks.yml` validates frontend build/lint/formatting and Rust tests on pushes to `main`.
- `build.yml` is the reusable macOS, Windows, and Linux packaging workflow.
- `release.yml` is manually triggered and creates a draft GitHub Release with platform bundles.

Automatic in-app updates and updater artifacts are disabled until a Kotha updater signing key is configured. Apple notarization and Windows code signing also require credentials owned by the Kotha maintainer.
