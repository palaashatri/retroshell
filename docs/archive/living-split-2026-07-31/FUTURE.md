# FUTURE — SLOPOS-I backlog & standing design constraints

> Backlog items are **not** the default agent focus unless the user prioritizes
> them. They must not displace UI SoT work or invent parallel status docs.
> Do not mark any of these done without QA evidence per [PROGRAM.md](PROGRAM.md).
> UI visual language lives in **[UI.md](UI.md)** — not here.
> Ops / current snapshot: **[HANDOFF.md](HANDOFF.md)**.
>
> **Gaps vs GNOME/KDE and the phased fix plan live in [MATURITY.md](MATURITY.md).**
> This file is the short backlog index; MATURITY is the gap register.

## Standing design constraint (applies to ALL UI work, now and future)

When designing or changing any UI, follow [UI.md](UI.md) references plus these HIG
links. Where they conflict, prefer the classic-Mac / helloSystem spirit:

- Apple macOS HIG — https://developer.apple.com/design/human-interface-guidelines/designing-for-macos
- helloSystem UX guidelines — https://hellosystem.github.io/docs/developer/ux-guidelines.html
- elementary HIG — https://docs.elementary.io/hig

Core implications already relevant to current work:

- **One global menu bar** owned by the shell (never per-app in-window menus).
- **Root-level dock** owned by the shell session, not by any app.
- Consistent control metrics, spacing, and focus behavior across apps (via
  `slopos-kit` / SDK paint).
- Trademark-safe branding (**SLOPOS** / **SLOPOS-I**, never Apple marks).
- Prefer **removing fake UI** over adding more decorative menus
  ([MATURITY.md](MATURITY.md) Phase A).

## Maturity headline

SLOPOS-I ≈ **15–25%** of a daily-driver DE vs GNOME/KDE. It is a real compositor +
classic-Mac shell slice with five partial apps — not a peer product. See the
scorecard and Phases A–E in [MATURITY.md](MATURITY.md).

## Feature backlog (maps to MATURITY phases)

### Phase A — honesty / ship proof
1. **Stage 4 DoD** — clean Arch + Ubuntu layered install; ISO boot (`qa/stage-4.md`).
2. **Wire or remove decorative menus** (shell + apps).
3. **Defect J on UTM** — re-prove clicks with an input injector.
4. **Rename leftovers** — e.g. `RetroBus` → `SloposBus`; optional host path/key names.

### Phase B — integration spine
5. **Defect H** — real `slopos-bus` transport (theme notify + round-trip tests).
6. **Live settings apply** — file watch and/or bus (file-based theme already works).
7. **System clipboard + primary selection**.
8. **Sync shell ↔ compositor workspaces**.
9. **Standard portals** — implement or proxy `xdg-desktop-portal`.

### Phase C — compositor / session
10. DRM client buffer presentation / scanout.
11. **XWayland on DRM path**.
12. Interactive move/resize/maximize for xdg clients.
13. **PAM** (or equivalent) lock — not plaintext conf password.
14. PipeWire screencast via portal.

### Phase D — app suite
15. Finder list view, open-with, rename, wired View/Edit.
16. Settings: NM Wi-Fi UI, display modeset that hits compositor.
17. TextEdit: portal file chooser + system clipboard.
18. Terminal: wire copy/paste/zoom; visible tab bar.
19. App Store: remove/update + optional HTTP catalog.
20. More first-party utilities **or** documented Flatpak escape hatch (browser).

### Phase E — visual kit parity (parallel; detail in UI.md)
21. **Spotlight polish** — file search, icons in results (`qa/v0.2.0/` MVP).
22. **Chicago/Geneva-class bitmap fonts**.
23. **Full System7Components control port** — checkbox, radio, slider, alert.
24. **Animated desktop backgrounds** (notes in `archive/ANIMATED-BACKGROUNDS.md`).
25. Graphite / icon fidelity (ongoing [UI.md](UI.md) gap list).

## Notes / dependencies

- Overlay / wallpaper features need shell-owned layer surfaces (already partly
  landed via `SLOPOS_LAYER_SHELL_CHROME=1`). Keep HDR/VRR compositor paths intact
  while iterating UI.
- Kit `Widget::draw` stubs mean many “widget ports” must wire through
  `slopos-sdk::draw_widget` to show up on screen.
- Phase D compounds after Phase B (clipboard/portals). Do not expand the app suite
  with more façades.
