# FUTURE — RetroShell backlog & standing design constraints

> Backlog items are **not yet started** and are out of scope for the current
> stages (0–4). They are recorded here so they are not lost. Do not mark any of
> these done without QA evidence per the honesty contract in [PROGRAM.md](PROGRAM.md).

## Standing design constraint (applies to ALL UI work, now and future)

When designing or changing any UI, follow these human-interface guidelines. Where
they conflict, prefer the classic-Mac / helloSystem spirit of the project:

- Apple macOS HIG — https://developer.apple.com/design/human-interface-guidelines/designing-for-macos
- helloSystem UX guidelines — https://hellosystem.github.io/docs/developer/ux-guidelines.html
- elementary HIG — https://docs.elementary.io/hig

Core implications already relevant to current work:
- **One global menu bar** owned by the shell (never per-app in-window menus).
- **Root-level dock** owned by the shell session, not by any app.
- Consistent control metrics, spacing, and focus behavior across apps (via `retro-kit`).

## Feature backlog (requested 2026-07-30)

1. **Spotlight-like global search** — system-wide launcher/search (apps, files,
   settings), invoked by a global shortcut; overlay surface owned by the shell.
2. **Multiple themes** — beyond the existing `themes/` (graphite, platinum,
   high-contrast, oled-graphite): a user-selectable theme system with a settings UI.
3. **Animated desktop backgrounds** — wallpapers from GIF / video (and possibly
   live/shader) sources, in addition to static images. Ties into the
   desktop/wallpaper surface (see the layer-shell chrome rework).

## Notes / dependencies
- (1) and (3) both want the shell to own real root-level surfaces (Spotlight = an
  overlay layer surface; animated wallpaper = the background layer surface), so
  they are cleaner to build **after** the layer-shell chrome rework lands.
