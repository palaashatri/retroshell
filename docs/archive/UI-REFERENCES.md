# UI References — Classic Macintosh / System 7 Canon

**Status:** Active design source of truth  
**Last Updated:** 2026-07-31  
**Scope:** Visual language only (kit + SDK canvas paint). Does **not** authorize regressing HDR/VRR, DRM, or compositor roadmap work.

SLOPOS-I’s UI must look as close as practical to Classic Mac / System 7 mockups and open reimplementations — **without** Apple trademarked logos or product names.

---

## Canonical references

### Code (port into Rust canvas painters)

| Source | Role |
|--------|------|
| [Calculable/System7Components](https://github.com/Calculable/System7Components) | **Primary paint recipes** — SwiftUI System 7 borders, frames, 3D buttons, menus, text fields, icons |
| [Kelsidavis/System7](https://github.com/Kelsidavis/System7) | Full OS reimplementation — patterns, resources, behavior reference (not a direct UI kit drop-in) |

### Figma (visual target)

| Source | Role |
|--------|------|
| [Classic Macintosh UI Kit (Community)](https://www.figma.com/community/file/1392611044307310359/classic-macintosh-ui-kit) | Community kit — chrome, controls, spacing |
| [Classic Macintosh UI Kit design](https://www.figma.com/design/LGMlwNCoVdakZxDBvPKg1W/Classic-Macintosh-UI-Kit--Community-?node-id=0-1) | Detailed frames / components |
| [System 7–like UI Kit](https://www.figma.com/design/8LqAFnsUxWQd4XeT6fPUEa/System-7--Apple-MacOS-7--like-UI-Kit--Community-?node-id=1-2) | Additional System 7–style reference |

---

## Hard constraints

1. **No Apple trademarks in shipping UI**
   - No rainbow Apple menu mark
   - No “Macintosh”, “Mac OS”, “Finder” as product branding in user-facing strings where avoidable (existing shell names like the file manager app may remain functional labels; do not add Apple logos)
   - Use SLOPOS glyph / “SLOPOS” system menu instead of Apple menu art

2. **Polish stays in paint path**
   - Implement look in `slopos-kit` + `slopos-sdk` canvas drawing
   - Do not rip out HDR/VRR, adaptive sync, or compositor protocol work to chase pixels

3. **Roadmap docs remain authoritative for non-UI systems**
   - `docs/PROGRAM.md`, stage tasks, HDR/VRR notes stay valid
   - UI polish must not claim those systems are “done” or rewrite their acceptance criteria

4. **Honesty**
   - Screenshots prove polish; code comments and docs must not claim kit-parity until VM evidence exists

---

## System7Components palette (light) — adopted

From `Assets.xcassets/Colors` in System7Components:

| Token | Hex | RGB |
|-------|-----|-----|
| Background | `#FFFFFF` | 255, 255, 255 |
| Foreground | `#000000` | 0, 0, 0 |
| Gray100 | `#EFEFEF` | 239, 239, 239 |
| Gray200 | `#DADADA` | 218, 218, 218 |
| Gray300 | `#A5A5A5` | 165, 165, 165 |
| Gray400 | `#868686` | 134, 134, 134 |
| Gray500 | `#666666` | 102, 102, 102 |
| Lavender100 | `#DADAFC` | 218, 218, 252 |
| Lavender200 | `#B3B3F9` | 179, 179, 249 |
| Lavender300 | `#8787B0` | 135, 135, 176 |
| Lavender500 | `#545483` | 84, 84, 131 |

---

## Port map (Swift → Rust)

| System7Components | SLOPOS-I |
|-------------------|------------|
| `System7Border.system73DBorder` | `draw_system7_3d_border` |
| `System73DButtonStyle` | multi-layer `draw_beveled_rect` |
| `System7Frame` / header grips | `draw_classic_titlebar` / `draw_window` |
| `System7FileSymbol` / icons | fixed-size `draw_*_icon` helpers (original art) |
| `System7Menu` | menu bar + popup painters |

---

## Related docs

- [`DESIGN-CLASSIC-MAC.md`](DESIGN-CLASSIC-MAC.md) — design language summary (status tracks kit parity)
- [`THEME-RENDERING-IMPLEMENTATION.md`](THEME-RENDERING-IMPLEMENTATION.md) — theme wiring
- [`qa/ui-polish/`](qa/ui-polish/) — visual evidence for this polish pass
