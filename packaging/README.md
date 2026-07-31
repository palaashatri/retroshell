# SLOPOS-I session packaging

These files register SLOPOS-I as a Wayland desktop session. The production
session is fail-closed: `slopos-session` supervises `slopos-compositor` and
`slopos-shell`; it never substitutes labwc, Sway, or another window manager.

## Installed runtime topology

```text
display manager / tty
└── start-slopos-i
    └── slopos-session
        ├── slopos-compositor
        │   └── private WAYLAND_DISPLAY=wayland-N
        └── slopos-shell
            └── first-party application processes
```

`slopos-compositor` publishes the exact private client socket through
`$XDG_RUNTIME_DIR/slopos-client-wayland-display`. The supervisor launches the
shell only after that socket exists and is a live Unix socket. Shell-launched
applications inherit only the private socket.

## Required binaries

Build and install:

- `slopos-session`
- `slopos-compositor`
- `slopos-shell`
- `finder`, `textedit`, `terminal`, `settings`, `appstore`
- `start-slopos-i`

```bash
cargo build --release --workspace --locked
sudo ./install.sh --prefix /usr
```

For session-file-only installation:

```bash
./scripts/install-session-files.sh --dry-run --prefix /usr
sudo ./scripts/install-session-files.sh --prefix /usr
```

## Backend selection

The backend is explicit:

```bash
# Bare-metal DRM/KMS session
SLOPOS_BACKEND=drm start-slopos-i

# Nested development under an existing graphical host
SLOPOS_BACKEND=nested start-slopos-i

# Protocol/headless development
SLOPOS_BACKEND=headless start-slopos-i
```

If the selected backend cannot initialize, the session exits with an error.
There is no automatic compositor fallback.

In nested mode, the host compositor should see one SLOPOS compositor output
window. SLOPOS shell/application clients must connect to the private socket
owned by `slopos-compositor`, not directly to the host socket.

## Environment variables

| Variable | Purpose |
| --- | --- |
| `SLOPOS_BACKEND` | `drm`, `nested`, or `headless` |
| `SLOPOS_COMPOSITOR_WAIT_SECS` | Private-socket readiness timeout (default 12) |
| `SLOPOS_HOST_WAYLAND_DISPLAY` | Host socket retained only by a nested backend |
| `SLOPOS_CLIENT_WAYLAND_DISPLAY` | Private compositor socket passed to clients |
| `SLOPOS_KEEP_DISPLAY` | Preserve `DISPLAY` for clients only when explicitly required |
| `SLOPOS_OUTPUTS_LAYOUT` | Compositor output arrangement configuration |
| `XDG_RUNTIME_DIR` | Runtime directory containing private socket/readiness file |

## Session files

| Source | Typical destination |
| --- | --- |
| `packaging/slopos-i-wayland.desktop` | `/usr/share/wayland-sessions/slopos-i.desktop` |
| `packaging/slopos-i.desktop` | `/usr/share/xsessions/slopos-i.desktop` |
| `scripts/start-slopos-i` | `/usr/bin/start-slopos-i` |
| `packaging/slopos-i.service` | `/usr/lib/systemd/user/slopos-i.service` |

A nested host compositor is a development transport only. It is not part of
the SLOPOS-I production window-management architecture.
