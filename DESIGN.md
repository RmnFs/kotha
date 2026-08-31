---
name: Kotha
description: A calm, logo-led desktop instrument for dependable speech-to-text.
colors:
  text: "var(--color-text)"
  background: "var(--color-background)"
  surface: "var(--color-surface)"
  surface-raised: "var(--color-surface-raised)"
  border: "var(--color-border)"
  muted: "var(--color-mid-gray)"
  brand-action: "var(--color-background-ui)"
  brand-green: "var(--color-brand-green)"
  brand-earth: "var(--color-brand-earth)"
  brand-vermilion: "var(--color-brand-vermilion)"
  warning: "var(--color-warning)"
  error: "var(--color-error)"
typography:
  title:
    fontFamily: "Avenir Next, Avenir, ui-sans-serif, -apple-system, BlinkMacSystemFont, Segoe UI, sans-serif"
    fontSize: "16px"
    fontWeight: 600
    lineHeight: "24px"
  body:
    fontFamily: "Avenir Next, Avenir, ui-sans-serif, -apple-system, BlinkMacSystemFont, Segoe UI, sans-serif"
    fontSize: "15px"
    fontWeight: 400
    lineHeight: "24px"
  control:
    fontFamily: "Avenir Next, Avenir, ui-sans-serif, -apple-system, BlinkMacSystemFont, Segoe UI, sans-serif"
    fontSize: "14px"
    fontWeight: 600
    lineHeight: "20px"
  label:
    fontFamily: "Avenir Next, Avenir, ui-sans-serif, -apple-system, BlinkMacSystemFont, Segoe UI, sans-serif"
    fontSize: "12px"
    fontWeight: 600
    lineHeight: "16px"
    letterSpacing: "0.08em"
rounded:
  md: "6px"
  lg: "8px"
  xl: "12px"
  2xl: "16px"
  overlay: "24px"
  pill: "9999px"
spacing:
  xs: "4px"
  sm: "8px"
  md: "12px"
  lg: "16px"
  xl: "20px"
  2xl: "24px"
components:
  button-primary:
    backgroundColor: "{colors.brand-action}"
    textColor: "#ffffff"
    typography: "{typography.control}"
    rounded: "{rounded.xl}"
    padding: "5px 16px"
  button-secondary:
    backgroundColor: "{colors.surface-raised}"
    textColor: "{colors.text}"
    typography: "{typography.control}"
    rounded: "{rounded.xl}"
    padding: "5px 16px"
  input-default:
    backgroundColor: "{colors.surface-raised}"
    textColor: "{colors.text}"
    typography: "{typography.control}"
    rounded: "{rounded.lg}"
    padding: "8px 12px"
  settings-group:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.text}"
    rounded: "{rounded.2xl}"
    padding: "12px 16px"
  nav-active:
    backgroundColor: "{colors.brand-action}"
    textColor: "#ffffff"
    typography: "{typography.control}"
    rounded: "{rounded.xl}"
    padding: "8px 12px"
  badge-primary:
    backgroundColor: "{colors.brand-vermilion}"
    textColor: "#ffffff"
    typography: "{typography.label}"
    rounded: "{rounded.pill}"
    padding: "4px 12px"
  recording-overlay:
    backgroundColor: "{colors.background}"
    textColor: "{colors.text}"
    rounded: "{rounded.overlay}"
    height: "40px"
---

# Design System: Kotha

## Overview

**Creative North Star: "The Paper-and-Ink Instrument"**

Kotha is a focused desktop instrument: warm paper and charcoal establish a quiet working ground, while the approved calligraphic wave/K supplies the human voice. The identity is expressive in the wordmark and recording signal, then restrained everywhere the user configures a task.

The system is compact, border-led, and operational. Familiar rounded controls, one calm settings column, and a narrow logo-anchored sidebar keep attention on configuration and state. Vermilion marks the active task; forest and earth support hover, progress, and the wave from speech to text.

**Key Characteristics:**

- Warm paper surfaces paired with ink-like text and dividers.
- A forest-to-earth-to-vermilion brand movement inherited from the approved Kotha artwork.
- Compact controls with generous enough targets, clear focus, and restrained motion.
- Flat surfaces at rest; depth is reserved for transient UI.
- Kotha's runtime contracts remain visually invisible and functionally stable.

**The Wordmark Leads Rule.** Preserve the approved wave, calligraphic K, terminal accent, lettering proportions, and color movement. Do not redraw the brand from generic type or substitute a stock microphone mark.

## Colors

The palette behaves like ink on warm paper in light mode and warm light on charcoal in dark mode. Semantic aliases in `src/styles/theme.css` are the source of truth and switch as a complete paired set.

### Primary

- **Grounded Action** (`--color-background-ui`; light `#96392d`, dark `#a84435`): primary buttons, selected navigation, and checked controls. Its solid field identifies the current task.
- **Voice Vermilion** (`--color-brand-vermilion`; light `#b74732`, dark `#df765e`): the logo endpoint, recording-state emphasis, progress, and focused brand detail.

### Secondary

- **Forest Ink** (`--color-brand-green`; light `#315b3d`, dark `#83a28a`): the opening of the wordmark movement and quiet positive or hover emphasis.
- **Earthen Bridge** (`--color-brand-earth`; light `#835f3f`, dark `#c49a6b`): connective brand color, field hover, and the middle waveform bars.

### Neutral

- **Writing Ink** (`--color-text`; light `#182019`, dark `#f4efe7`): primary text and icons.
- **Warm Paper** (`--color-background`; light `#f4eee4`, dark `#171a17`): application ground.
- **Quiet Sheet** (`--color-surface`; light `#fffaf2`, dark `#20251f`): sidebar and grouped settings containers.
- **Raised Sheet** (`--color-surface-raised`; light `#fffdf9`, dark `#282e27`): controls and menus.
- **Ink Hairline** (`--color-border`; light `#d8cdbd`, dark `#3c453b`): section boundaries, cards, and control outlines.
- **Muted Annotation** (`--color-mid-gray`; light `#626a61`, dark `#aab1a7`): descriptions, helper text, and inactive detail.

### Named Rules

**The Active Field Rule.** Use a solid Grounded Action field for the selected or primary state; use translucent Forest, Earth, or Vermilion only for subordinate feedback.

**The Paired Theme Rule.** Every reusable color enters through a shared semantic alias with a tested light and dark value. Component-local brand hex values are drift.

**The Signal Is Semantic Rule.** Warning and error use their theme tokens; brand vermilion is not a substitute for destructive meaning.

## Typography

**Display Font:** Not used; the approved Kotha wordmark is an image asset, not typeset display copy.  
**Body Font:** Avenir Next, with Avenir and the platform sans-serif stack as fallbacks.  
**Label Font:** The body stack at compact sizes and stronger weights.

**Character:** Typography is calm and native-adjacent, with a compact hierarchy suited to a settings utility. Weight, muted color, and spacing create hierarchy; oversized headings do not.

### Hierarchy

- **Title** (600, 16px, 24px): dialog titles, model names, and the strongest in-flow headings.
- **Body** (400, 15px, 24px): application default and longer explanatory text.
- **Control** (600, 14px, 20px): buttons, navigation, fields, and setting titles.
- **Label** (600, 12px, 16px, 0.08em when grouping): compact metadata and settings-group labels.

### Named Rules

**The Utility Scale Rule.** Keep application hierarchy between 12px and 16px; the wordmark supplies display presence while the interface stays scan-friendly.

## Layout

The settings window is a full-height flex shell with a fixed narrow sidebar (`176px`) and one scrollable content column. The sidebar carries the wordmark in an `80px` header band, then stacks `40px`-minimum navigation items. Content uses `20px` horizontal and `24px` vertical insets with `20px` gaps between major blocks.

Spacing follows a compact 4px-derived rhythm. Settings rows use `12px 16px` padding and a `56px` minimum height; related rows share one outlined group rather than repeating separate cards.

**The One Working Column Rule.** Keep one primary settings column in the remaining window; do not convert operational preferences into a dashboard grid.

## Elevation & Depth

The application is flat by default. Warm tonal differences and ink-like borders separate the page, sidebar, settings groups, fields, menus, dialogs, and recording overlay. Shadows appear only on transient overlays such as tooltips and toasts; the persistent settings architecture and recording pill do not float.

### Shadow Vocabulary

- **Transient Lift** (`box-shadow: 0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1)`): tooltips and toasts only.
- **Selected Whisper** (`box-shadow: 0 1px 2px 0 rgb(0 0 0 / 0.05)`): the active navigation field.

### Named Rules

**The Flat Instrument Rule.** Persistent surfaces separate through tone and hairlines. Do not add glass blur, gradients, or resting drop shadows to make containers feel important.

## Shapes

Corners are compact and continuous: fields use gently curved `8px` corners, buttons and navigation use `12px`, grouped settings and dialogs use `16px`, and the resting recording surface uses a `24px` pill-like corner. Full pills are reserved for switches, status badges, and circular icon controls.

Borders are single-pixel hairlines. The recording overlay changes from a `24px` resting corner to `18px` while working and `16px` when expanded, so its geometry communicates state without changing material.

**The Radius Follows Scale Rule.** Increase radius with container scale; do not apply full-pill geometry to rectangular settings fields or cards.

## Components

### Buttons

- **Shape:** Compact rounded rectangle (`12px`) with a one-pixel border and semibold label.
- **Primary:** Grounded Action field with white text and `5px 16px` medium padding; hover shifts to Voice Vermilion.
- **Secondary:** Raised Sheet with an Ink Hairline; hover gains a pale Forest tint and Forest border.
- **Hover / Focus:** Color and border transitions run at `150ms`; focus uses a visible two-pixel brand ring.
- **Ghost:** Transparent until interaction, with a muted tonal fill and brand-colored border on hover.

### Chips

- **Style:** Compact full pills with `4px 12px` padding and a `12px` medium-weight label.
- **State:** Primary status uses a solid brand field; secondary metadata uses a muted translucent surface.

### Cards / Containers

- **Corner Style:** Grouped settings and model cards use gently rounded `16px` corners.
- **Background:** Quiet Sheet at rest; active model cards use a faint Voice Vermilion tint.
- **Shadow Strategy:** None at rest; a one-pixel Ink Hairline establishes the edge.
- **Internal Padding:** Repeating rows use `12px 16px`; vertical groups use `10px` gaps.

### Inputs / Fields

- **Style:** Raised Sheet, Ink Hairline, `8px` corners, semibold `14px` text, and `8px 12px` default padding.
- **Focus:** Border changes to Voice Vermilion with a two-pixel translucent ring.
- **Hover:** Border changes to Earthen Bridge before focus.
- **Error / Disabled:** Error uses the semantic error token; disabled fields reduce opacity and keep the same structural border.

### Navigation

The `176px` sidebar is a Quiet Sheet bounded by one logical-end hairline. Items are `40px` minimum height with `12px` corners. Inactive items are quiet text with a faint Forest hover; the active item becomes a solid Grounded Action field with white text. Focus includes a brand ring and a surface-colored offset so it remains visible against either state.

### Toggle Switch

The switch is a `44px × 24px` full pill with a `20px` white thumb. Off uses a muted translucent track; on uses Grounded Action. Focus expands as a clear brand ring.

### Recording Overlay

The overlay is the signature state component: a flat, near-opaque themed surface with a one-pixel hairline, compact fixed state widths, and a `40px` control row. The waveform explicitly moves from Forest through Earth to Vermilion. Width and corner morphs use a `460ms` emphasized easing curve; micro-state color changes stay around `150ms`, and waveform response is linear at `80ms`.

## Do's and Don'ts

### Do:

- **Do** source all reusable color from `src/styles/theme.css` and keep light, dark, system, and explicit overrides aligned.
- **Do** let the Kotha wordmark anchor the sidebar and preserve generous clear space around it.
- **Do** reserve the solid Grounded Action field for the primary or active task.
- **Do** retain visible keyboard focus and familiar desktop-control behavior.
- **Do** use the Forest–Earth–Vermilion sequence when a component represents voice movement, especially the waveform.

### Don't:

- **Don't** redraw, recolor, crop, or typeset a replacement for the approved Kotha artwork.
- **Don't** reintroduce the discarded generic pink utility palette or component-local brand hex values.
- **Don't** turn the settings window into a card dashboard, marketing surface, or decorative showcase.
- **Don't** add gradients, glass blur, or resting shadows to persistent controls and containers.
- **Don't** change runtime identifiers, storage paths, settings keys, shortcut IDs, or backend events as incidental visual work.
