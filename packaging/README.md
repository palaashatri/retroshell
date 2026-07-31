# SLOPOS-I session packaging (Phase A)

> Status pointer: living SoT is [`docs/SLOPOS-I.md`](../docs/SLOPOS-I.md);
> VM ops in §§7–8. Session file names use `slopos-i` / `start-slopos-i`
> (product **SLOPOS-I**).

Files here register SLOPOS-I as a **Wayland session** so a display manager
(GDM, SDDM, LightDM, etc.) can start it from the greeter. The session script
starts a compositor, then `slopos-shell`.

| File | Role |
|------|------|
| `slopos-i.desktop` | Generic XDG session desktop entry (`Type=Application`, `DesktopNames=SLOPOS-I`) |
| `slopos-i-wayland.desktop` | Same entry intended for `/usr/share/wayland-sessions/` |
| `../scripts/start-slopos-i` | Session launcher (compositor + shell) |

## Prerequisites

Build and install (or put on `PATH`) at least:

- `slopos-shell`
- `slopos-compositor` (preferred) **or** `labwc` (fallback)
- `start-slopos-i` (this repo’s `scripts/start-slopos-i`)

Example from a release build tree:

```bash
cargo build --release -p slopos-shell -p slopos-compositor
sudo install -Dm755 target/release/slopos-shell /usr/local/bin/slopos-shell
sudo install -Dm755 target/release/slopos-compositor /usr/local/bin/slopos-compositor
sudo install -Dm755 scripts/start-slopos-i /usr/local/bin/start-slopos-i
```

`start-slopos-i` also resolves binaries from `target/release` / `target/debug`
relative to the script when you run it from a git checkout without installing.

## Install session files for display managers

Wayland greeters load sessions from `/usr/share/wayland-sessions/` (system) or
sometimes `/usr/local/share/wayland-sessions/`.

Preferred (dry-run first; default `PREFIX=/usr/local`):

```bash
./scripts/install-session-files.sh --dry-run
sudo ./scripts/install-session-files.sh --prefix /usr/local
# system-wide greeter paths often need --prefix /usr
sudo ./scripts/install-session-files.sh --prefix /usr
```

This installs:

| Destination under `$PREFIX` | Source |
|-----------------------------|--------|
| `share/wayland-sessions/slopos-i.desktop` | `packaging/slopos-i-wayland.desktop` |
| `share/xsessions/slopos-i.desktop` | `packaging/slopos-i.desktop` |
| `bin/start-slopos-i` | `scripts/start-slopos-i` |
| `lib/systemd/user/slopos-i.service` | `packaging/slopos-i.service` |

Manual equivalent:

```bash
# System-wide (typical)
sudo install -Dm644 packaging/slopos-i-wayland.desktop \
  /usr/share/wayland-sessions/slopos-i.desktop

# Or install the generic name
sudo install -Dm644 packaging/slopos-i.desktop \
  /usr/share/wayland-sessions/slopos-i.desktop

# Ensure the Exec= target is on PATH for the greeter user session
sudo install -Dm755 scripts/start-slopos-i /usr/local/bin/start-slopos-i
```

After install, log out and pick **SLOPOS-I** on the greeter (session menu).
Cold path: login → `start-slopos-i` → compositor → `slopos-shell`.

Some DMs also accept a session under `/usr/share/xsessions/`; SLOPOS-I is a
**Wayland** session — prefer `wayland-sessions` only.

### Optional: absolute Exec= path

If the greeter’s environment is minimal and does not include `/usr/local/bin`:

```desktop
Exec=/usr/local/bin/start-slopos-i
TryExec=/usr/local/bin/start-slopos-i
```

Edit the installed `.desktop` file accordingly.

## Manual run (no greeter)

```bash
# Default: slopos-compositor if available, else labwc
./scripts/start-slopos-i

# Force labwc (e.g. known missing DRI3 / nested Docker)
SLOPOS_FORCE_LABWC=1 ./scripts/start-slopos-i
# or
SLOPOS_COMPOSITOR=labwc ./scripts/start-slopos-i

# Require slopos-compositor (no silent labwc fallback)
SLOPOS_COMPOSITOR=slopos-compositor ./scripts/start-slopos-i
```

## Compositor selection (honest fallback)

| Condition | Behavior |
|-----------|----------|
| Default + `slopos-compositor` on PATH | Start it, wait briefly; if it dies → **labwc** |
| `SLOPOS_FORCE_LABWC` set | **labwc** only |
| `SLOPOS_COMPOSITOR=labwc` | **labwc** only |
| `SLOPOS_COMPOSITOR=slopos-compositor` | **slopos-compositor** only (exit if it fails) |
| Nested X (`DISPLAY` set, no existing `WLR_BACKENDS`) | labwc started with `WLR_BACKENDS=x11` and software render allowed |
| Bare metal / DRM seat | labwc default backend; `WLR_BACKENDS` left unset unless you export it |

**DRI3 note:** Nested environments (Docker-on-mac, plain Xvfb without DRI3) often
cannot keep `slopos-compositor` alive. That is an environment limit. The script
prints that fact and falls back to labwc instead of pretending the Smithay
path is running. Check `$XDG_RUNTIME_DIR/slopos-compositor.log` (or the tail
printed at fallback).

Docker’s `docker-entrypoint.sh` implements the same preference order for the
noVNC/dev image; host sessions should use `start-slopos-i` instead of the
container entrypoint.

## Environment variables

| Variable | Meaning |
|----------|---------|
| `SLOPOS_COMPOSITOR` | `labwc` or `slopos-compositor` |
| `SLOPOS_FORCE_LABWC` | Non-empty → force labwc |
| `SLOPOS_COMPOSITOR_WAIT_SECS` | Startup grace period (default `3`) |
| `SLOPOS_OUTPUTS_LAYOUT` | Multi-monitor layout blob; re-exported when set for `display_arrange` |
| `SLOPOS_KEEP_DISPLAY` | Set to `1` to keep `DISPLAY` when exec’ing the shell |
| `WLR_BACKENDS` | If already set, passed through to labwc as-is |
| `WLR_RENDERER_ALLOW_SOFTWARE` | Used with nested x11 labwc (default `1` when nested) |
| `WAYLAND_DISPLAY` | Set by the script after the compositor is up |
| `XDG_RUNTIME_DIR` | Required for Wayland sockets; default `/run/user/$(id -u)` |

## Uninstall

```bash
sudo rm -f /usr/share/wayland-sessions/slopos-i.desktop
sudo rm -f /usr/local/bin/start-slopos-i
# Optionally remove slopos-shell / slopos-compositor from /usr/local/bin
```
