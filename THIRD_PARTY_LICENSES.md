# Third-party notices

SLOPOS-I first-party code is MIT-licensed. The project is built on third-party
Rust crates and Linux system components; those works remain owned by their
respective authors and are distributed under their own licenses.

The repository-level `LICENSE` does **not** relicense dependencies, fonts,
firmware, Linux, Mesa, Smithay, Wayland protocols, or other external software.
Binary distributors must preserve all notices required by those components.

## Dependency policy

CI should run:

```bash
cargo deny check licenses advisories bans sources
```

The policy is defined in `deny.toml`. Before publishing a release, generate a
complete machine-derived attribution bundle from the exact locked dependency
graph, for example with `cargo-about` or `cargo-license`, and ship it beside the
binary packages. Do not maintain a guessed hand-written crate inventory.

## System substrate

SLOPOS-I may integrate with externally supplied Linux services such as the
kernel, DRM/KMS drivers, Mesa, PipeWire, NetworkManager, BlueZ, UDisks2,
systemd/logind, XWayland, and fonts. They are not authored or owned by the
SLOPOS-I project and are not covered by the SLOPOS-I MIT grant.
