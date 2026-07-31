# SLOPOS-I UI — Source of Truth

**Status:** In progress — closer than early flat builds, **not** kit-parity.  
**Evidence:** [`qa/ui-polish/`](qa/ui-polish/) (polish) and [`qa/v0.2.0/`](qa/v0.2.0/) (Spotlight + theme bar).  
**Paint code:** `crates/slopos-sdk/src/lib.rs` (`draw_widget`, window/menu/dock/icons).  
**Kit:** `crates/slopos-kit` — many `Widget::draw` methods are still **stubs**; do not
assume implementing kit `draw()` alone makes pixels appear.

This is the **only** living UI/design doc. Session notes and “theme 100% complete”
write-ups were archived under [`archive/`](archive/).

---

## Goal

Match Classic Macintosh / System 7 look from open kits and Figma — **without**
Apple trademarked logos or names — while keeping HDR/VRR and stage roadmap work
intact.

### Canonical references

| Source | Use |
|--------|-----|
| [Calculable/System7Components](https://github.com/Calculable/System7Components) | Primary paint recipes (borders, frame, 3D buttons, colors) |
| [Kelsidavis/System7](https://github.com/Kelsidavis/System7) | OS reimpl / patterns / behavior |
| [Figma Classic Mac UI Kit (community)](https://www.figma.com/community/file/1392611044307310359/classic-macintosh-ui-kit) | Visual target |
| [Figma LGMlwNCoVdakZxDBvPKg1W](https://www.figma.com/design/LGMlwNCoVdakZxDBvPKg1W/Classic-Macintosh-UI-Kit--Community-?node-id=0-1) | Detailed chrome |
| [Figma System 7–like kit](https://www.figma.com/design/8LqAFnsUxWQd4XeT6fPUEa/System-7--Apple-MacOS-7--like-UI-Kit--Community-?node-id=1-2) | Extra System 7 reference |

### Hard constraints

- No rainbow Apple mark; use SLOPOS glyph / **“SLOPOS”** system menu.
- Polish stays in kit/SDK canvas path — do not rip compositor/HDR/VRR.
- Never claim polish without fresh non-blank `qa/ui-polish/` screenshots on the VM
  (UTM: sway+grim; see [HANDOFF.md](HANDOFF.md)).
- Product strings say **SLOPOS-I**; chromeless desktop title must remain
  **`SLOPOS-I Desktop`** (special-cased in `draw_window`).

---

## System7Components palette (light)

| Token | Hex |
|-------|-----|
| Background | `#FFFFFF` |
| Foreground | `#000000` |
| Gray100–500 | `#EFEFEF` … `#666666` |
| Lavender100 | `#DADAFC` (focused title rail) |

**Graphite (dark):** not “light chrome on dark wallpaper.” Menu, window chrome,
dock, and icon faces must all shift to graphite surfaces with light text and dark
bevels. Preference file: `~/.config/slopos-i/settings.conf` (`theme=dark`).

---

## Port map (Swift → Rust)

| System7Components | SLOPOS-I |
|-------------------|----------|
| `system73DBorder` | `draw_system7_3d_border` |
| `System73DButtonStyle` | multi-layer `draw_beveled_rect` |
| `System7Frame` header | `draw_classic_titlebar` |
| File/app symbols | fixed 32×32 `draw_labeled_icon` / `draw_*_icon` |
| Overlay panels | `Panel` + SDK `draw_widget` (Spotlight) |

---

## Honest current gaps (2026-07-31)

Looking at `qa/ui-polish/` + `qa/v0.2.0/`:

1. **Still not kit-parity** — multi-edge chrome improved; dark Graphite is
   fuller-stack but menu/chrome can still read wrong (too light / uneven).
2. **Icons** — distinct trademark-safe glyphs exist; still schematic vs
   System7Components pixel art. Hard Disk / folder confusion can still happen.
3. **Typography** — block/sans painter, not Chicago/Geneva-class bitmaps.
4. **Controls kit** — checkbox / radio / slider / alert assets not ported.
5. **Kit draw stubs** — TextField/ListView/etc. `draw()` empty; Spotlight works
   only because SDK paints the tree.
6. **Program / DE gaps** — Stage 4 VM DoD unverified; Defect H broken; Defect J not
   re-proven on UTM. Broader gaps vs GNOME/KDE (portals, PAM, XWayland-on-DRM,
   decorative menus, thin suite) + fix phases: [MATURITY.md](MATURITY.md).
   See also [PROGRAM.md](PROGRAM.md).

### Done recently (proven in screenshots)

- Docs SoT consolidated; project renamed to SLOPOS-I in UI strings (“SLOPOS”
  menu, “SLOPOS HD”, About SLOPOS-I).
- Multi-layer bevels + black 3D window border.
- Title grips / close / zoom; lavender focused rail.
- Fixed-size icons (no full-width gray column bands from shadow misuse).
- Theme-aware Graphite menu/dock/icons + desktop nameplates.
- Spotlight overlay paints with query/results (`qa/v0.2.0/`).
- Finder dir detection uses path metadata (fewer false “document” folders).

---

## How to iterate

1. Change paint in `slopos-sdk` (and kit only when the SDK path will call it).
2. Build on UTM VM ([HANDOFF.md](HANDOFF.md)); set `SLOPOS_LAYER_SHELL_CHROME=1`
   + software GL.
3. Capture with sway+grim into `qa/ui-polish/` (replace in place).
4. Update **this file’s gap list** — not a new session markdown.

Superseded docs live in [`archive/`](archive/) (do not resurrect as parallel SoT).
