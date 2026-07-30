# HANDOFF — continue RetroShell (updated 2026-07-30)

> You are a fresh coding agent taking over mid-effort. Read this top to bottom,
> then `docs/PROGRAM.md` (honesty contract) and
> `docs/tasks/stage-2b-layer-shell-chrome.md` (the active workstream).
> **Honesty contract governs everything:** a task/stage is done only when its
> acceptance command passes on the real VM, evidenced by a transcript or
> screenshot — never by reading code or self-scoring. Do not mark anything
> "verified" you have not actually run.

## 1. What this project is
RetroShell is a classic-Mac-styled Linux **desktop environment** in Rust (Cargo
workspace). Own Wayland compositor (`crates/retro-compositor`, smithay), own shell
(`crates/retro-shell`), a widget kit (`crates/retro-kit`), an app SDK
(`crates/retro-sdk`, winit+wgpu), and first-party apps (`apps/*`).

## 2. ENVIRONMENT — this is current; ignore any older Windows/VBox notes
- **Host:** macOS (Apple Silicon). **VM:** UTM VM named `Ubuntu`, **aarch64**,
  Ubuntu 26.04, at **192.168.64.15**, user `ubuntu`/`ubuntu` (passwordless sudo).
- Start VM: `/Applications/UTM.app/Contents/MacOS/utmctl start Ubuntu`. Find IP if
  it changed: `arp -a | grep 192.168.64`. No qemu-guest-agent (utmctl ip fails).
- SSH: `ssh -i ~/.ssh/retroshell_utm ubuntu@192.168.64.15` (host key checking off).
- The VM is fully provisioned (rust, wayland/drm/seatd/libinput/gbm/egl dev libs,
  fonts-dejavu, xvfb, imagemagick, sway, grim, libxkbcommon-x11-0, 4G swap).
  User is in `video,render,input` groups (the `seat` group does not exist here).
- **Sync + build workflow** (edit on host, build on VM):
  ```bash
  rsync -az --exclude target --exclude .git --exclude docs/screenshots \
    -e "ssh -i ~/.ssh/retroshell_utm" ./ ubuntu@192.168.64.15:/home/ubuntu/retroshell/
  ssh -i ~/.ssh/retroshell_utm ubuntu@192.168.64.15 \
    'cd ~/retroshell && source ~/.cargo/env && cargo build --release -p <crate>'
  ```
  ⚠️ Any **dependency change** (Cargo.toml) triggers a ~15 min feature
  re-unification rebuild; code-only changes are fast (20 s–2 min).
- **You cannot build on the macOS host** (wayland/DRM). Always build on the VM.
  Do not trust subagents that claim "cargo check passes" on the host.

## 3. Running & screenshots on the VM
- **Software GL is mandatory** for wgpu clients (virtio hardware GL fails):
  `export LIBGL_ALWAYS_SOFTWARE=1 GALLIUM_DRIVER=llvmpipe`.
- **Compositor on real DRM/KMS** (headless over SSH):
  `export XDG_RUNTIME_DIR=/run/user/1000 LIBSEAT_BACKEND=seatd` then
  `./target/release/retro-compositor` → gets seat0 + /dev/dri/card0 + 1280×800
  modeset + spawns retro-shell.
- **Screenshots:** the compositor's SIGUSR1 offscreen dump
  (`crates/retro-compositor/src/screenshot.rs`) is **BLOCKED** — this GLES driver
  rejects `glReadPixels` for every format. Two working methods instead:
  - **Layer-shell UI** (the shell desktop): run under **sway headless + grim** —
    see the exact recipe in `docs/tasks/stage-2b-layer-shell-chrome.md` and the
    `qa-layer-desktop.png` capture. (Xvfb cannot host layer-shell.)
  - **Plain winit UI:** Xvfb + `import -window root` (winit uses X11 when DISPLAY
    is set and WAYLAND_DISPLAY is unset).

## 4. What was done this session (branch `docs/program-design`)
Re-audited Cursor's Stage 2 (it was overclaimed "VERIFIED"); fixed and evidenced:
- **Fonts fixed** — baseline bearing/ascent in `retro-render` + bitmap descenders
  in `retro-sdk`. Verified (`docs/screenshots/qa-shell-xvfb.png`).
- **Global-menu-only** — removed the legacy in-window `MenuBar` from `retro-sdk`.
- **Audit corrections** — `docs/qa/stage-2.md` retracts the false "VERIFIED";
  deleted misleading old screenshots; added `docs/FUTURE.md` backlog + HIG
  constraints; added `docs/tasks/stage-2b-layer-shell-chrome.md`.
- **Layer-shell rework (the big one) — Phase 2b rendering DONE:**
  - `retro_sdk::RawSurfaceRenderer` — wgpu surface from raw wl_display/wl_surface.
  - `retro_sdk::UiRuntime` — backend-agnostic render+input core (mirrors the winit
    `AppHandler`); `tick()` drives `ShellDesktop::update()` (dock/notifications).
  - `crates/retro-shell/src/layer_desktop.rs` — wlr-layer-shell **background**
    surface driver; paints `ShellDesktop` fullscreen. Gated behind
    `RETROSHELL_LAYER_SHELL_CHROME`; winit path untouched when unset.
  - **Verified under sway+grim** (`docs/screenshots/qa-layer-desktop.png`):
    fullscreen root-level desktop, full-width menu bar, correct fonts, desktop
    icons, Finder window, dock. This is the milestone.
- Compositor also got: a `new_toplevel` shell-fullscreen attempt (keyed on
  app_id — does NOT fire, winit doesn't set app_id; superseded by layer-shell)
  and the (blocked) SIGUSR1 screenshot tool.

Commits (newest first): `2552fc6` polish (chromeless + dock) · `119abc5` evidence
· `a7d5e37` layer_desktop · `db41171` UiRuntime · `1bd616c` RawSurfaceRenderer ·
`441b6a6` scope · `aea7014` fonts/menu/audit.

## 5. NEXT STEPS (priority order) — the active workstream
1. **Verify under our OWN compositor.** So far the layer desktop is proven under
   *sway*, not `retro-compositor`. Run `retro-compositor` on the VM with
   `RETROSHELL_LAYER_SHELL_CHROME=1` in its env (spawned shell inherits it) and
   confirm it composites the layer-shell background surface. If not, fix the DRM
   path's layer-surface compositing (`session_drm.rs` `collect_render_elements`
   likely doesn't include layer surfaces — the nested `main.rs` render_frame does;
   port that). This is the real product integration and is UNVERIFIED.
2. **Wire input (Phase 2b-iii).** `layer_desktop.rs` accepts wl_pointer events but
   does NOT route them yet (see the `let _ = (state, event)` in the WlPointer
   Dispatch). Route Motion→`runtime.pointer_moved(surface_x,surface_y)` and
   Button→`runtime.pointer_button(map(button), pressed, time_ms)`; add wl_keyboard
   (xkb) → `runtime.key(...)`. Model on `crates/retro-shell/src/bin/retro-lock.rs`.
3. **Phase 3: split exclusive chrome.** Currently one background layer holds
   everything, so a maximized app could cover the menu/dock. Split menu bar → a
   `top` layer (exclusive_zone = menu_h) and dock → a `bottom` layer
   (exclusive_zone = dock_h); keep wallpaper+icons on background. Then DELETE the
   throwaway `crates/retro-shell/src/layer_shell_client.rs` (a PoC that maps gray
   placeholder surfaces; still fired from `startup()` ~line 598 — remove that call)
   and un-stub `chrome_protocol.rs::should_paint_kit_chrome`.
4. **Minor polish:** menu-bar item spacing looks tight (words nearly touch);
   clock/live content only updates on wayland events (blocking_dispatch) — add a
   frame-callback or timer tick for liveness.
5. **Re-QA Stage 2 functional claims** (lock unbypass, Super+O) on the reworked
   build and update `docs/qa/stage-2.md` with real evidence.

## 6. Gotchas
- Some docs (e.g. `docs/qa/stage-2.md` before this session) had **CRLF** line
  endings from the prior Windows work — use Write, not line-based Edit, if a match
  mysteriously fails.
- `RETROSHELL_LAYER_SHELL_CHROME` is overloaded: it gates BOTH the new
  `layer_desktop` path (in `run()`) AND the old throwaway `layer_shell_client`
  (in `startup()`). Remove the latter in Phase 3.
- Keep the winit default path working — it's the fallback and how apps render.
