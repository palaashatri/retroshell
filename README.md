# SLOPOS-I

SLOPOS-I is an experimental, sovereign Linux desktop environment written in
Rust. It combines classic Macintosh/System 7 interaction principles with a
modern Wayland compositor, shell, application SDK, system apps, local Vision
features, user-controlled fonts, and planned SLOPOS Spaces.

It is under active development and is **not yet a KDE/GNOME-class daily-driver**.
See `TRUTH.md` for the current audited status and known defects.

## Documentation

The repository deliberately has only three Markdown files:

- [`README.md`](README.md) — introduction and quick start;
- [`AGENTS.md`](AGENTS.md) — complete architecture, product requirements,
  engineering rules, implementation order and acceptance criteria;
- [`TRUTH.md`](TRUTH.md) — current audit, evidence, maturity and defect ledger.

Do not add parallel roadmaps, reports, matrices or session-summary Markdown.
Raw QA artifacts belong under `artifacts/qa/`.

## Architecture

```text
slopos-session
├── slopos-compositor
│   └── private Wayland socket
├── slopos-shell
├── Finder / Settings / TextEdit / Terminal / App Store / Preview
└── session services such as slopos-visiond
```

In nested development, the host compositor should see one outer SLOPOS window;
all shell and application surfaces are managed inside it by `slopos-compositor`.
No production labwc/Sway fallback is intended.

## Workspace

Core crates include:

- `slopos-session`, `slopos-compositor`, `slopos-shell`;
- `slopos-render`, `slopos-kit`, `slopos-sdk`, `slopos-bus`;
- `slopos-fonts`;
- `slopos-vision`, `slopos-vision-protocol`, `slopos-vision-client`,
  `slopos-visiond`.

First-party applications live under `apps/`.

## Build

A current stable Rust toolchain and the Linux development dependencies listed in
`packaging/deps/` are required.

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace
cargo build --release --workspace
```

The exact build status of the archived 2026-08-01 snapshot is not independently
verified in this repository cleanup pass; consult `TRUTH.md` and record fresh
results there.

## Run a nested development session

```bash
cargo build --release -p slopos-session -p slopos-compositor -p slopos-shell
SLOPOS_BACKEND=nested ./scripts/start-slopos-i
```

On software-rendered VMs, these may be required:

```bash
LIBGL_ALWAYS_SOFTWARE=1 GALLIUM_DRIVER=llvmpipe \
  SLOPOS_BACKEND=nested ./scripts/start-slopos-i
```

Backend names and command-line behavior must be checked against
`./target/release/slopos-compositor --help` and `scripts/start-slopos-i` in the
current source. Do not rely on this quick start as proof that a backend works.

## Install and package

The tree contains Arch, Debian, session and ISO packaging under `packaging/`,
plus `install.sh`. Clean installation, display-manager login and ISO boot must
be verified on target VMs before release claims are made.

## SLOPOS Vision

`slopos-vision` contains an early local OCR and subject-segmentation engine.
Model files are not silently downloaded at runtime. The daemon/client/Preview
integration is still incomplete in the audited snapshot; see `TRUTH.md`.

## Licensing

First-party source and original assets are MIT-licensed:

```text
Copyright (c) 2026 Palaash Atri
```

Third-party crates, Linux components, fonts, codecs and model weights retain
their own licenses. See `LICENSE`, `COPYRIGHT`, `THIRD_PARTY_LICENSES.txt`,
`deny.toml`, and `models/vision/manifest.toml`.
