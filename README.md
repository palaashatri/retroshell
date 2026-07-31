# SLOPOS-I

Classic Mac / System 7–styled Linux desktop environment written in Rust
(Wayland compositor, shell, toolkit, SDK, and first-party apps).

> Formerly **RetroShell**. Product, crates, binaries, env vars, session files,
> and docs are **SLOPOS-I** / `slopos-*`.

## Documentation

**One living doc:** **[docs/SLOPOS-I.md](docs/SLOPOS-I.md)**  
(honesty, status, stages, UI, maturity gaps + fix plan, VM ops)

Evidence / tasks / archive: [`docs/README.md`](docs/README.md)

## Quick start (Linux VM)

```bash
cargo build --release -p slopos-compositor -p slopos-shell
SLOPOS_LAYER_SHELL_CHROME=1 ./scripts/start-slopos-i
```

On UTM/virtio-gpu also set `LIBGL_ALWAYS_SOFTWARE=1 GALLIUM_DRIVER=llvmpipe`.

Session packaging: [packaging/README.md](packaging/README.md) · `./install.sh`

## Naming

| Kind | Form |
|------|------|
| Product | **SLOPOS-I** |
| Config dir | `~/.config/slopos-i` |
| Env prefix | `SLOPOS_*` |
| Crates / bins | `slopos-*` |
| Session desktop | `slopos-i.desktop` |
| Launcher | `start-slopos-i` |
| System menu | **SLOPOS** |
