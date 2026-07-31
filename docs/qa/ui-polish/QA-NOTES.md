# UI Polish QA — System7-faithful pass

**Date:** 2026-07-31  
**Environment:** UTM Ubuntu aarch64 @ 192.168.64.15  
**Capture:** sway headless + grim  
**References:** [`docs/UI-REFERENCES.md`](../../UI-REFERENCES.md)

## What changed (this pass)

| Area | Change |
|------|--------|
| Palette | System7Components Gray/Lavender/FG/BG constants |
| Bevels | Multi-layer `draw_beveled_rect` (System73DButton stack) |
| Windows | `draw_system7_3d_border` + Frame-style title grips / close / zoom |
| Menu bar | White face + 1px black bottom rule |
| Icons | Fixed 32×32 paint (kills gray column bands); unique per-app glyphs |
| Dock | Same labeled icon map |

## Screenshots

| File | Proves |
|------|--------|
| `01-desktop.png` | Light desktop: no column bands, distinct app icons, white menu, gripped title bar |
| `02-desktop-dark.png` | Dark theme still paints with System7 chrome |
| `03-spotlight.png` | Spotlight still works atop polished desktop |

## Honest gaps vs Figma / System7Components

- Bitmap Chicago/Geneva fonts not ported (glyph renderer still block font)
- Checkbox/radio/slider/alert PNG assets not ported yet
- Some desktop icon metaphors still schematic (not kit pixel art)
- Dock is RetroShell chrome (System 7 had no modern dock) — styled, not removed
- HDR/VRR untouched (by design)

## Reproduce

```bash
export WLR_BACKENDS=headless WLR_LIBINPUT_NO_DEVICES=1
export LIBGL_ALWAYS_SOFTWARE=1 GALLIUM_DRIVER=llvmpipe
export RETROSHELL_LAYER_SHELL_CHROME=1
sway -c /tmp/sway-headless.conf &
./target/release/retro-shell &
sleep 7 && grim docs/qa/ui-polish/01-desktop.png
```

**Tests:** 317 lib tests passed on VM after this pass.
