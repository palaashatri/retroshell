# RetroShell session packaging (Phase A)

Files here register RetroShell as a **Wayland session** so a display manager
(GDM, SDDM, LightDM, etc.) can start it from the greeter. The session script
starts a compositor, then `retro-shell`.

| File | Role |
|------|------|
| `retroshell.desktop` | Generic XDG session desktop entry (`Type=Application`, `DesktopNames=RetroShell`) |
| `retroshell-wayland.desktop` | Same entry intended for `/usr/share/wayland-sessions/` |
| `../scripts/start-retroshell` | Session launcher (compositor + shell) |

## Prerequisites

Build and install (or put on `PATH`) at least:

- `retro-shell`
- `retro-compositor` (preferred) **or** `labwc` (fallback)
- `start-retroshell` (this repo’s `scripts/start-retroshell`)

Example from a release build tree:

```bash
cargo build --release -p retro-shell -p retro-compositor
sudo install -Dm755 target/release/retro-shell /usr/local/bin/retro-shell
sudo install -Dm755 target/release/retro-compositor /usr/local/bin/retro-compositor
sudo install -Dm755 scripts/start-retroshell /usr/local/bin/start-retroshell
```

`start-retroshell` also resolves binaries from `target/release` / `target/debug`
relative to the script when you run it from a git checkout without installing.

## Install session files for display managers

Wayland greeters load sessions from `/usr/share/wayland-sessions/` (system) or
sometimes `/usr/local/share/wayland-sessions/`.

```bash
# System-wide (typical)
sudo install -Dm644 packaging/retroshell-wayland.desktop \
  /usr/share/wayland-sessions/retroshell.desktop

# Or install the generic name
sudo install -Dm644 packaging/retroshell.desktop \
  /usr/share/wayland-sessions/retroshell.desktop

# Ensure the Exec= target is on PATH for the greeter user session
sudo install -Dm755 scripts/start-retroshell /usr/local/bin/start-retroshell
```

After install, log out and pick **RetroShell** on the greeter (session menu).
Cold path: login → `start-retroshell` → compositor → `retro-shell`.

Some DMs also accept a session under `/usr/share/xsessions/`; RetroShell is a
**Wayland** session — prefer `wayland-sessions` only.

### Optional: absolute Exec= path

If the greeter’s environment is minimal and does not include `/usr/local/bin`:

```desktop
Exec=/usr/local/bin/start-retroshell
TryExec=/usr/local/bin/start-retroshell
```

Edit the installed `.desktop` file accordingly.

## Manual run (no greeter)

```bash
# Default: retro-compositor if available, else labwc
./scripts/start-retroshell

# Force labwc (e.g. known missing DRI3 / nested Docker)
RETROSHELL_FORCE_LABWC=1 ./scripts/start-retroshell
# or
RETROSHELL_COMPOSITOR=labwc ./scripts/start-retroshell

# Require retro-compositor (no silent labwc fallback)
RETROSHELL_COMPOSITOR=retro-compositor ./scripts/start-retroshell
```

## Compositor selection (honest fallback)

| Condition | Behavior |
|-----------|----------|
| Default + `retro-compositor` on PATH | Start it, wait briefly; if it dies → **labwc** |
| `RETROSHELL_FORCE_LABWC` set | **labwc** only |
| `RETROSHELL_COMPOSITOR=labwc` | **labwc** only |
| `RETROSHELL_COMPOSITOR=retro-compositor` | **retro-compositor** only (exit if it fails) |
| Nested X (`DISPLAY` set, no existing `WLR_BACKENDS`) | labwc started with `WLR_BACKENDS=x11` and software render allowed |
| Bare metal / DRM seat | labwc default backend; `WLR_BACKENDS` left unset unless you export it |

**DRI3 note:** Nested environments (Docker-on-mac, plain Xvfb without DRI3) often
cannot keep `retro-compositor` alive. That is an environment limit. The script
prints that fact and falls back to labwc instead of pretending the Smithay
path is running. Check `$XDG_RUNTIME_DIR/retro-compositor.log` (or the tail
printed at fallback).

Docker’s `docker-entrypoint.sh` implements the same preference order for the
noVNC/dev image; host sessions should use `start-retroshell` instead of the
container entrypoint.

## Environment variables

| Variable | Meaning |
|----------|---------|
| `RETROSHELL_COMPOSITOR` | `labwc` or `retro-compositor` |
| `RETROSHELL_FORCE_LABWC` | Non-empty → force labwc |
| `RETROSHELL_COMPOSITOR_WAIT_SECS` | Startup grace period (default `3`) |
| `RETROSHELL_KEEP_DISPLAY` | Set to `1` to keep `DISPLAY` when exec’ing the shell |
| `WLR_BACKENDS` | If already set, passed through to labwc as-is |
| `WLR_RENDERER_ALLOW_SOFTWARE` | Used with nested x11 labwc (default `1` when nested) |
| `WAYLAND_DISPLAY` | Set by the script after the compositor is up |
| `XDG_RUNTIME_DIR` | Required for Wayland sockets; default `/run/user/$(id -u)` |

## Uninstall

```bash
sudo rm -f /usr/share/wayland-sessions/retroshell.desktop
sudo rm -f /usr/local/bin/start-retroshell
# Optionally remove retro-shell / retro-compositor from /usr/local/bin
```
