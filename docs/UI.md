# RetroShell UI — Source of Truth

**Status:** In progress — closer than before, **not** kit-parity.  
**Evidence:** [`qa/ui-polish/`](qa/ui-polish/) only.  
**Paint code:** `crates/retro-sdk/src/lib.rs` (+ widgets in `crates/retro-kit`).

This is the **only** living UI/design doc. Session notes and “theme complete” write-ups were archived.

---

## Goal

Match Classic Macintosh / System 7 look from open kits and Figma — **without** Apple trademarked logos or names — while keeping HDR/VRR and stage roadmap work intact.

### Canonical references

| Source | Use |
|--------|-----|
| [Calculable/System7Components](https://github.com/Calculable/System7Components) | Primary paint recipes (borders, frame, 3D buttons, colors) |
| [Kelsidavis/System7](https://github.com/Kelsidavis/System7) | OS reimpl / patterns / behavior |
| [Figma Classic Mac UI Kit (community)](https://www.figma.com/community/file/1392611044307310359/classic-macintosh-ui-kit) | Visual target |
| [Figma LGMlwNCoVdakZxDBvPKg1W](https://www.figma.com/design/LGMlwNCoVdakZxDBvPKg1W/Classic-Macintosh-UI-Kit--Community-?node-id=0-1) | Detailed chrome |
| [Figma System 7–like kit](https://www.figma.com/design/8LqAFnsUxWQd4XeT6fPUEa/System-7--Apple-MacOS-7--like-UI-Kit--Community-?node-id=1-2) | Extra System 7 reference |

### Hard constraints

- No rainbow Apple mark; use Retro glyph / “Retro” menu.
- Polish stays in kit/SDK canvas path — do not rip compositor/HDR/VRR.
- Never claim polish without fresh `qa/ui-polish/` screenshots.

---

## System7Components palette (light)

| Token | Hex |
|-------|-----|
| Background | `#FFFFFF` |
| Foreground | `#000000` |
| Gray100–500 | `#EFEFEF` … `#666666` |
| Lavender100 | `#DADAFC` (focused title rail) |

**Graphite (dark):** not “light chrome on dark wallpaper.” Menu, window chrome, dock, and icon faces must all shift to graphite surfaces with light text and dark bevels.

---

## Port map (Swift → Rust)

| System7Components | RetroShell |
|-------------------|------------|
| `system73DBorder` | `draw_system7_3d_border` |
| `System73DButtonStyle` | multi-layer `draw_beveled_rect` |
| `System7Frame` header | `draw_classic_titlebar` |
| File/app symbols | fixed 32×32 `draw_labeled_icon` / `draw_*_icon` |

---

## Honest current gaps (as of latest screenshots)

Looking at `qa/ui-polish/`:

1. **Still not kit-parity** — multi-edge chrome improved; dark Graphite is fuller-stack but not finished.
2. **Icons** — better per-app glyphs; still schematic vs System7Components pixel art.
3. **Typography** — block/sans, not Chicago/Geneva.
4. **Controls kit** — checkbox/radio/slider/alert not ported.
5. **Functionality vs `docs/`** — Stages 0–3 verified; Stage 4 VM unverified; defect H broken; UTM button re-QA pending. See [PROGRAM.md](PROGRAM.md).

### Done recently (proven in screenshots)

- Docs consolidated to README / PROGRAM / UI / HANDOFF / FUTURE (+ tasks/qa/specs)
- Multi-layer bevels + black 3D window border
- Title grips / close / zoom
- Fixed-size icons (no column bands)
- Theme-aware Graphite menu/dock/icons + desktop nameplates
- Spotlight overlay paints (`qa/v0.2.0/`)
- Finder dir detection uses metadata (fewer false “document” folders)

---

## How to iterate

1. Change paint in `retro-sdk`.
2. Build on UTM VM (`HANDOFF.md`).
3. Capture with sway+grim into `qa/ui-polish/`.
4. Update **this file’s gap list** — not a new session markdown.

Superseded docs live in [`archive/`](archive/) (do not resurrect as parallel SoT).
