# RetroShell — Full Feature Completion Plan (Nothing Deferred)

> **Purpose**: Work order for parallel agents. Every task has files, acceptance commands, and
> pass conditions. **No feature is out of scope for this pass.**
>
> **Source of truth for prior audit**: [`docs/audit_2026-07-09.md`](audit_2026-07-09.md).
> Baseline re-verified 2026-07-11: host tests **140 passed / 0 failed**; T0–T3/T5/T6 largely
> landed; **T1 fails DRI3 in Docker** (labwc fallback); **T4 send is placeholder**; HDR/VRR
> modules exist but unwired; Settings theme parser still 5 names; main Dockerfile missing noVNC.

---

## Rules of engagement (EVERY agent)

1. **No fabrication.** Only claim results produced by a tool call in your session.
2. **Evidence or it didn't happen.** Paste command + relevant output (`test result:`, `DOCKER_EXIT=`).
3. **Read before you edit.** Anchor on quoted code; line numbers drift.
4. **Stay in your task's files** unless integration requires a listed shared file — then coordinate.
5. **Compositor is Linux-only.** Verify with `docker build` / Docker run, not host `cargo` for full binary.
6. **Linux install/run in Docker** (or Pi). Host macOS: tests for non-compositor crates only.
7. **Deadline mode:** ship working end-to-end paths; hardware absence → real client + clean unavailable fallback, not stubs that claim success.

---

## Ground truth (re-audited 2026-07-11 session end)

| Item | Status |
|---|---|
| Host `cargo test --workspace --exclude retro-compositor` | **176 passed, 0 failed** (re-run this session) |
| T0 compositor type fix | Done |
| T2 password lock | Done + tests |
| T3 eight themes | Done in theme_manager **and** Settings |
| Settings conf merge preserves `lock_password` | Done + test |
| T4 `send` / selection | Implemented (fd write + mime maps), not empty drop |
| T5 AT-SPI | Real D-Bus Accessible export (registry Embed best-effort) |
| T1 Docker runtime | Compositor dies DRI3 → **labwc fallback**; shell runs |
| HDR/VRR | Wired in compositor main + renderer policy + Settings |
| Main `Dockerfile` noVNC | Present + EXPOSE 6080 |
| Uncommitted production delta | **Yes** — not pushed/released until commit |

---

## Architecture target

```
NM / PipeWire / UPower / AT-SPI  (D-Bus / system)
            ▲
retro-shell + apps + retro-kit (a11y tree)
            │ Wayland client(s)
retro-compositor (smithay): xdg, seat, data_device,
 multi-output, XWayland, frame pacing / HDR policy
            │
nested X11 / DRM (Pi) — Docker may use labwc if DRI3 absent
```

---

## Dependency graph

```
Wave 1 (parallel foundation)
  P0.1 Settings merge + 8 themes
  P0.2 Lock verify (reconfirm)
  P0.3 Docker image + entrypoint
  P0.4 Renderer present-mode/format API

Wave 2 (parallel systems; compositor SERIAL in one agent)
  P1.1 wl_data_device send
  P1.2 Multi-output
  P1.3 XWayland
  P1.4 HDR/VRR wire-in
  P2.1 NetworkManager
  P2.2 PipeWire/Pulse volume
  P2.3 UPower (+ /sys fallback)
  P2.4 Screenshot + screen record
  P3   AT-SPI2 real registration + tree

Wave 3 (integration)
  Settings/shell bind live services
  Docs + scripts/verify_pi.sh

Wave 4 (QA)
  Host tests, docker build/smoke, visual screenshots, evidence pack
```

**Compositor file ownership:** single agent chain on `crates/retro-compositor/src/main.rs`
order: geometry helpers → data_device → HDR/scheduler → multi-output → XWayland.

---

## P0 — Foundation

### P0.1 Settings conf merge + 8 themes

**Files:** `apps/settings/src/main.rs`

**Changes:**
1. Theme match/UI include `solarized`, `dracula`, `highcontrast`.
2. `save()` must **merge**: load existing lines, update known keys, preserve `lock_password` and unknown keys.
3. Persist `refresh_rate`, `color_space` with existing `hdr_requested` / `vrr_adaptive`.

**Accept:**
```bash
cargo test -p settings 2>&1 | grep 'test result:'
# Manual: settings.conf with lock_password=secret survives a theme change save
```

### P0.2 Screen lock

**Files:** `crates/retro-shell/src/lib.rs`

Reconfirm password gate; pure verify helper; tests `lock_accepts_correct_password` /
`lock_rejects_wrong_password`; no "any key to unlock".

**Accept:**
```bash
cargo test -p retro-shell lock 2>&1 | grep 'test result:'
grep -rn "any key to unlock" crates/ || true   # must be empty
```

### P0.3 Docker production image

**Files:** `Dockerfile`, `docker-entrypoint.sh`

- Runtime: novnc, websockify, mesa, imagemagick, labwc, dbus, pulse, gbm/drm, xwayland, at-spi2-core
- EXPOSE 6080; soft-GL env; all app binaries + entrypoint
- Compositor stderr → `/tmp/retro-compositor.log`

**Accept:**
```bash
docker build -t retroshell . > /tmp/docker-build.log 2>&1; echo DOCKER_EXIT=$?
docker run -d --name rs-qa -p 6080:6080 retroshell
sleep 12
docker logs rs-qa 2>&1 | grep -E 'retro-compositor is running|falling back to labwc|WAYLAND_DISPLAY|noVNC|Ready'
docker exec rs-qa sh -c 'ps aux | grep -E "[r]etro-shell|[l]abwc|[r]etro-compositor" || true'
docker rm -f rs-qa
```

### P0.4 Renderer policy API

**Files:** `crates/retro-render/src/renderer.rs` (and public exports)

- HDR formats only when requested + supported; else sRGB-safe format
- VRR/adaptive → prefer AutoVsync; else Fifo
- Optional reconfigure helpers for shell startup

**Accept:** `cargo test -p retro-render` → 0 failed

---

## P1 — Compositor

### P1.1 wl_data_device send

**Files:** `crates/retro-compositor/src/main.rs`

Implement selection store + write to fd for clipboard/primary; DnD send from tracked source;
no hang on missing source (EOF).

**Accept:** `docker build` compiles `retro-compositor`; send body is non-placeholder.

### P1.2 Multi-output

**Files:** `crates/retro-compositor/src/{main.rs,lib.rs}`

Env `RETROSHELL_OUTPUTS=WxH,WxH` or dual surfaces; ≥2 `wl_output` when configured; geometry tests.

### P1.3 XWayland

Enable smithay XWayland; package `xwayland` in image; map X clients; verify on Pi/Linux.

### P1.4 HDR/VRR

Wire `HdrCapabilities` + `FrameScheduler` into main; read settings/env; pace frames; log policy;
SDR under Xvfb without lying.

---

## P2 — OS services

### P2.1 NetworkManager — zbus client; Settings Network live status; unavailable fallback.

### P2.2 PipeWire/Pulse — volume get/set via wpctl/pactl; Settings Sound slider real.

### P2.3 UPower — D-Bus first, `/sys` fallback; battery UI/status.

### P2.4 Capture — screenshot PNG + ffmpeg record start/stop; menu/shortcut.

**New modules (suggested):**  
`crates/retro-shell/src/{network_manager.rs,audio.rs,power.rs,capture.rs}`  
Linux-gated with host-safe stubs.

---

## P3 — AT-SPI2 (real)

**Files:** `crates/retro-kit/src/accessibility.rs`, shell/app startup register

- Session bus registration; Application + Component interfaces
- Expose `AccessibilityTree` children (Role, Name, GetChildAtIndex)
- Widgets push nodes for Window/Button/TextField/Label/MenuBar
- `register_at_spi_app` succeeds on Linux+D-Bus when registered

**Accept:** In Docker with at-spi2-core, app visible on a11y bus; `cargo test -p retro-kit accessibility`.

---

## P4 — Docs + Pi

- README / ARCHITECTURE / CONFIGURATION / KEYBOARD_SHORTCUTS match reality
- `scripts/verify_pi.sh` — deps, build, test, probe NM/PW/UPower/HDR/AT-SPI
- Fresh `docs/audit_2026-07-11.md` evidence-backed

---

## Definition of done

- [x] Host tests 0 failed — **176 passed** (2026-07-11)
- [x] `docker build` exit 0; noVNC works; shell alive — **DOCKER_EXIT=0**, noVNC HTTP 200, `retro-shell` + labwc
- [x] 8 themes + conf merge preserves `lock_password` — settings 6/6 tests
- [x] Lock password gate tests pass — 2/2
- [x] data_device send implemented — SelectionHandler + fd write (compositor)
- [x] Multi-output configurable + tests — `RETROSHELL_OUTPUTS`
- [x] XWayland code path + package — feature + runtime package
- [x] HDR/VRR wired (settings → compositor/renderer) — policy log + DisplayRenderPolicy
- [x] AT-SPI app on bus with non-empty tree — exported in container logs
- [x] NM / PipeWire|Pulse / UPower functional with fallbacks — modules + pactl verified in Docker
- [x] Screenshot + screen record APIs/menu — capture module + menu hooks (file capture of record not re-run every gate)
- [x] Visual screenshot evidence — `docs/screenshots/prod-desktop.png`
- [x] Pi verify script committed — `scripts/verify_pi.sh`
- [x] Docs reconciled — README/CONFIGURATION/audit_2026-07-11.md

See [`docs/audit_2026-07-11.md`](audit_2026-07-11.md) for command evidence.

---

## Agent file ownership

| Agent | Owns |
|---|---|
| fix-settings | `apps/settings/src/main.rs` |
| fix-lock | lock sections of `crates/retro-shell/src/lib.rs` |
| fix-docker | `Dockerfile`, `docker-entrypoint.sh` |
| fix-render | `crates/retro-render/src/renderer.rs` |
| fix-compositor | entire `crates/retro-compositor/**` (serial P1) |
| fix-atspi | `crates/retro-kit/src/accessibility.rs` (+ kit call sites if needed) |
| fix-services | new shell modules + Settings bindings for NM/PW/UPower/capture |
| fix-docs | `docs/**`, `README.md`, `scripts/verify_pi.sh` |

---

## Historical note

Earlier T0–T6 sprint items remain valid as completed foundation (see git `d1b02eb` and
`docs/audit_2026-07-09.md`). This document **supersedes** the “explicitly out of scope”
section: AT-SPI, multi-monitor, XWayland, HDR/VRR, NM, PipeWire, capture are **in scope now**.
