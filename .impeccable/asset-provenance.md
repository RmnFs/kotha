# Kotha Asset Provenance

This inventory covers the raster assets created or replaced by the Kotha rebrand. It deliberately excludes pre-existing release-note and sponsor imagery, which the rebrand neither changes nor claims.

## Authority and derivation

- **Visual authority:** user-supplied `/Users/rmn/Developer/Romanize/Kotha.jpeg` (2048×2048 RGB JPEG).
- **Deterministic producer:** `scripts/generate-kotha-assets.swift` removes the white matte, cleans compression noise, crops the approved artwork, isolates the wave/K mark, and produces brand, icon-master, and tray sources.
- **Platform icon producer:** `bun tauri icon src-tauri/icons/kotha-icon-master.png` derives the platform PNG, ICO, and ICNS families from the opaque master.
- **Required invariant:** preserve the approved calligraphic wave/K, terminal accent, lettering and spacing, and forest-green → earth-brown → vermilion movement. No generated file introduces a new brand concept.

## Shipping inventory

| Asset family | Files | Count | Provenance carrier |
| --- | --- | ---: | --- |
| In-app brand artwork | `public/brand/kotha-wordmark.png`, `public/brand/kotha-mark.png`, `public/brand/kotha-symbol.png` | 3 | Embedded PNG `impeccable:prompt` text chunk |
| App-icon master and platform PNGs | All PNG files under `src-tauri/icons/`, including Windows Store, iOS, Android, and adaptive-icon variants | 50 | Embedded PNG `impeccable:prompt` text chunk |
| Platform icon containers | `src-tauri/icons/icon.icns`, `src-tauri/icons/icon.ico` | 2 | Adjacent `.json` provenance sidecars |
| Tray and colored state rasters | `src-tauri/resources/handy.png`, `handy_warning.png`, `recording.png`, `transcribing.png`, and all `tray_*.png` theme/state variants | 12 | Embedded PNG `impeccable:prompt` text chunk |

The PNG inventory is verified with:

```bash
node /Users/rmn/.agents/skills/impeccable/scripts/embed-prompt.mjs \
  --scan public/brand src-tauri/icons src-tauri/resources
```

Expected result: `SCAN: 65 rasters, 0 missing`.

## Integration boundary

Asset filenames remain compatible with existing React imports, Tauri configuration, tray routing, package identity, bundle and storage locations, updater configuration, settings keys, shortcut IDs, commands, generated bindings, and backend events. The Kotha rebrand changes visible identity and raster contents only; it does not rename internal Handy contracts.
