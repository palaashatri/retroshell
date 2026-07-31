# v0.2.0 Visual QA Results

**Date:** 2026-07-31  
**Environment:** UTM `Ubuntu` aarch64 @ 192.168.64.15 (Ubuntu 26.04)  
**Capture method:** sway headless (`WLR_BACKENDS=headless`) + `grim`  
**Product name:** SLOPOS-I (docs/crates may still show older “RetroShell” strings in
archive only; living tree uses SLOPOS-I / `slopos-*` / `SLOPOS_*`).  
**Honesty note:** Earlier `docs/qa/v0.2.0/01-desktop.png` (263 bytes, blank) and the previous
QA write-up claiming “ready to release” without pixels are **invalid**. This file only
claims what the screenshots below prove. Living status: [SLOPOS-I.md](../../SLOPOS-I.md).

## What was wrong before

| Prior claim | Reality |
|---|---|
| Spotlight rendering complete | `TextField::draw` / `ListView::draw` are empty stubs; overlay was **not** in the canvas paint tree |
| Visual QA passed | `import -window root` produced empty 1-bit PNGs; no UI was captured |
| Ready to tag v0.2.0 | Spotlight was invisible; screenshots were lies |

## Build / tests (VM)

```
cargo test -p slopos-kit -p slopos-sdk -p slopos-shell --lib --release
→ 317 passed; 0 failed
cargo build --release -p slopos-shell
→ success
```

## Screenshots (evidence)

All under `docs/qa/v0.2.0/`:

| File | What it proves | Size |
|------|----------------|------|
| `01-desktop.png` | Desktop paints: menu bar, Finder (“SLOPOS HD”), desktop icons, dock | ~42 KB |
| `02-spotlight.png` | Spotlight visible with query `vol` → result **Volume — Sound** selected | ~39 KB |
| `03-spotlight-empty.png` | Spotlight card + placeholder + app suggestions list | ~44 KB |
| `04-desktop-dark.png` | `theme=dark` in `~/.config/slopos-i/settings.conf` darkens chrome/backdrop | ~42 KB |
| `05-spotlight-settings.png` | Query `Settings` → Settings app + WiFi Settings result | ~42 KB |

### How Spotlight was opened for screenshots

No `wtype`/`ydotool` on this VM. Overlay was forced once at startup via:

```bash
SLOPOS_QA_SPOTLIGHT=vol ./target/release/slopos-shell
```

That env hook is a QA aid only; production toggle remains **Super+Space** (unit-tested).

### Capture recipe (reproducible)

```bash
export XDG_RUNTIME_DIR=/run/user/$(id -u)
export WLR_BACKENDS=headless WLR_LIBINPUT_NO_DEVICES=1
export LIBGL_ALWAYS_SOFTWARE=1 GALLIUM_DRIVER=llvmpipe
export SLOPOS_LAYER_SHELL_CHROME=1
sway -c /tmp/sway-headless.conf &   # output * { resolution 1280x800 }
export SWAYSOCK=... WAYLAND_DISPLAY=wayland-1
./target/release/slopos-shell &
sleep 7
grim docs/qa/v0.2.0/01-desktop.png
```

## Feature verdict (v0.2.0 scope)

### Spotlight (B2c) — PASS (visually proven)
- Scrim + raised card + text field + results list paint through SDK `draw_widget`
- Query filtering works (`vol`, `Settings`)
- Selection highlight visible
- Unit tests: Super+Space toggle, char input, arrows, Escape, Enter activation

### Theme system — PASS (partial visual proof)
- Light Platinum desktop proven (`01-desktop.png`)
- Dark theme via `settings.conf` proven (`04-desktop-dark.png`)
- **Not proven this session:** live toggle inside Settings UI / hot-swap without restart

### Defect J (button clicks) — NOT RE-PROVEN on UTM
- Stage 2 previously marked button click PASS on Env B (`qa-layer-input-click.png`)
- This UTM session has no pointer-injection tool installed; clicks were **not** re-validated here
- Keyboard workflows (Spotlight, menus via unit tests) work
- **Honest status:** deferred / rely on Stage 2 evidence; do not claim new UTM click proof

## v0.2.0 checklist

| Item | Status | Evidence |
|------|--------|----------|
| Source builds on VM | PASS | release build log |
| Lib tests pass on VM | PASS | 317/317 |
| Desktop visible | PASS | `01-desktop.png` |
| Spotlight visible + results | PASS | `02` / `03` / `05` |
| Dark theme renders | PASS | `04-desktop-dark.png` |
| Button clicks on UTM | NOT RUN | no input injector |
| Old blank screenshots | REMOVED | replaced with real PNGs |

## Remaining after v0.2.0

1. Re-verify Defect J on UTM with `wtype`/`ydotool` or compositor test harness
2. Settings UI theme toggle screenshot (hot-swap)
3. Unique app icons (many still share the blue “A” monitor placeholder)
4. Tag/release only after this QA file stays honest

---

**v0.2.0 functional bar (bare minimum):** Spotlight + themed desktop are **visually proven** on the UTM VM.  
**Not claimed:** perfect polish, live Settings theme UI, or fresh UTM button-click proof.
