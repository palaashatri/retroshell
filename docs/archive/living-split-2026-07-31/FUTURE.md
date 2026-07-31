# FUTURE — RetroShell backlog & standing design constraints

> Backlog items are **not yet started** (or not stage-gated) and must not displace
> current stage / UI SoT work. Recorded so they are not lost. Do not mark any of
> these done without QA evidence per the honesty contract in [PROGRAM.md](PROGRAM.md).
> UI visual language lives in **[UI.md](UI.md)** — not here.

## Standing design constraint (applies to ALL UI work, now and future)

When designing or changing any UI, follow [UI.md](UI.md) references plus these HIG
links. Where they conflict, prefer the classic-Mac / helloSystem spirit:

- Apple macOS HIG — https://developer.apple.com/design/human-interface-guidelines/designing-for-macos
- helloSystem UX guidelines — https://hellosystem.github.io/docs/developer/ux-guidelines.html
- elementary HIG — https://docs.elementary.io/hig

Core implications already relevant to current work:
- **One global menu bar** owned by the shell (never per-app in-window menus).
- **Root-level dock** owned by the shell session, not by any app.
- Consistent control metrics, spacing, and focus behavior across apps (via `retro-kit`).

## Feature backlog

1. **Spotlight polish** — richer file search, icons in results (MVP already paints; see `qa/v0.2.0/`).
2. **Theme picker UX** — Settings UI for live theme swap (preference file already works).
3. **Animated desktop backgrounds** — GIF / video wallpapers (details formerly in `archive/ANIMATED-BACKGROUNDS.md`).
4. **Chicago/Geneva-class bitmap fonts** — replace block glyph painter for kit typography.
5. **Full System7Components control port** — checkbox, radio, slider, alert assets.

## Notes / dependencies

- Overlay / wallpaper features need shell-owned layer surfaces (already partly landed
  via layer-shell chrome). Keep HDR/VRR compositor paths intact while iterating UI.
