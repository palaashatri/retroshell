# Stage 4 — Distribution (layer-first installer + secondary ISO)

> **For executors:** read [docs/tasks/README.md](README.md) first. Do tasks in
> order. **Stage status: UNVERIFIED** — authored 2026-07-30 from the design spec
> (§4 Stage 4) and verbatim code grounding; not yet run. No task here is "done"
> until its Acceptance passes and the transcript is in
> [docs/qa/stage-4.md](../qa/stage-4.md).

**Goal (spec §4 Stage 4):** SLOPOS-I is a **desktop environment on a normal
Linux base**, so the **primary** delivery is layering it onto an existing distro;
a **bootable ISO** is a secondary convenience built from the *same* session files
so the two cannot drift.

**Definition of done (spec §4):**
1. On a **clean Arch VM** *and* a **clean Ubuntu-server VM**, the layered installer
   produces a **login-selectable SLOPOS-I session that reaches the desktop**.
2. The **ISO boots** a fresh VM straight into SLOPOS-I.
Transcripts + screenshots in [docs/qa/stage-4.md](../qa/stage-4.md).

## Grounding caveat (honesty contract)

Stage 1 is verified; Stages 2–3 have not run on the VM yet. Stage 4 packages and
ships whatever the build produces — it does **not** depend on Stage 2/3 runtime,
but a session that "reaches the desktop" is only as good as the compositor Stage 1
proved. So Stage 4's DoD is legitimately reachable now (Stage 1 painted Finder),
and it gets *better* as 2–3 land. Re-run Stage 4's VM tasks after 2–3 to confirm
no regression.

## Planner decisions (resolve spec §10 Q2 — executor must NOT re-decide)

1. **The layered `install.sh` is primary.** It runs on an already-installed
   Arch or Ubuntu system, installs deps, builds, installs binaries + session
   files, and (optionally) a greeter. The AUR `PKGBUILD` and the `.deb` are
   *native-package conveniences* built on the **same** dependency manifests
   (Task 4.1) — no second source of truth.
2. **Greeter for "login-selectable":** standardize on **`greetd` + `tuigreet`**
   (both packaged in Arch and Ubuntu, Wayland-native, minimal — works even on
   Ubuntu-server which ships no DM). The installer configures greetd to read
   `/usr/share/wayland-sessions/`. Installing the greeter is **opt-in**
   (`--with-greeter`) because a graphical machine may already have gdm/sddm; on
   headless Ubuntu-server you pass `--with-greeter`.
3. **Session files are shared, already exist, and are installed via the existing
   `scripts/install-session-files.sh`** — do not duplicate their content.
4. **ISO tool is `archiso`** (Arch-based live image), reusing Task 4.1's Arch dep
   list and the same session files.

## Current-state anchors (verbatim from grounding, 2026-07-30)

- Binary install (fresh-Arch script): `packaging/vm/arch-install.sh:125-130` —
  `install -Dm755 target/release/<bin> /usr/local/bin/<bin>` for
  `slopos-compositor, slopos-shell, finder, settings, textedit, terminal, appstore`
  and `scripts/start-slopos-i`.
- Session registration: `arch-install.sh:131` installs
  `packaging/slopos-i.desktop` → `/usr/share/wayland-sessions/slopos-i.desktop`.
- Session files in repo: `packaging/slopos-i.desktop`,
  `packaging/slopos-i-wayland.desktop`, `packaging/slopos-i.service`,
  `scripts/start-slopos-i` (232 lines; resolves binaries, prefers
  `slopos-compositor`, falls back to `labwc`).
- **Existing layered-install primitive:** `scripts/install-session-files.sh`
  installs (under `--prefix`, default `/usr/local`) the wayland-session, xsession,
  `bin/start-slopos-i`, and `lib/systemd/user/slopos-i.service`. **Reuse it.**
- No PKGBUILD, no `debian/`, no archiso profile, no non-VM `install.sh` exist yet
  (only the VM scripts + `install-session-files.sh`).
- Runtime deps (Arch, verbatim): `arch-install.sh:37-55` — see Task 4.1.
- 7 binaries produced by `cargo build --release --workspace`.

## Global constraints

- The installer must be **idempotent** (safe to re-run) and must not assume a
  fresh disk — it runs on a *running* system (unlike `arch-install.sh`, which
  partitions `/dev/sda` and must **never** be reused here).
- One dependency source of truth (Task 4.1). `install.sh`, PKGBUILD, `.deb`, and
  ISO all read it.
- Every VM DoD task captures a screenshot + transcript into `docs/qa/stage-4.md`.

---

### Task 4.0 — Re-ground: confirm session files + primitives exist   [UNVERIFIED]

Precondition:
```bash
git rev-parse --abbrev-ref HEAD
```

Steps:
1. Confirm the shared artifacts are present:
   ```bash
   ls packaging/slopos-i.desktop packaging/slopos-i-wayland.desktop \
      packaging/slopos-i.service scripts/start-slopos-i scripts/install-session-files.sh
   ```
2. Confirm the binary list the installer must copy:
   ```bash
   grep -n 'install -Dm755' packaging/vm/arch-install.sh
   ```

Acceptance:
```bash
test -f scripts/install-session-files.sh && test -f scripts/start-slopos-i && \
test -f packaging/slopos-i-wayland.desktop && echo STAGE4-BASELINE-CONFIRMED
```
→ expect: `STAGE4-BASELINE-CONFIRMED`.

DO NOT:
- Edit code here — verification only.
- Confuse `packaging/vm/arch-install.sh` (fresh-disk installer, partitions
  `/dev/sda`) with the layered installer you are about to build. They are
  different tools.

Commit: _none (verification only)._

---

### Task 4.1 — Canonical dependency manifests   [UNVERIFIED]

Precondition:
```bash
grep -n 'pacstrap' packaging/vm/arch-install.sh   # find the Arch dep list (~L37-55)
```

Files: Create `packaging/deps/arch.txt` and `packaging/deps/ubuntu.txt`.

Steps:
1. `packaging/deps/arch.txt` — one package per line, the **runtime** subset of
   `arch-install.sh:37-55` (exclude base-system-only: `base`, `linux`,
   `linux-firmware`, `grub`, `efibootmgr`, `base-devel` if you keep a separate
   build-deps note). Include:
   ```text
   wayland
   wayland-protocols
   libxkbcommon
   libinput
   seatd
   libdrm
   mesa
   vulkan-icd-loader
   vulkan-swrast
   vulkan-tools
   libdisplay-info
   pixman
   dbus
   at-spi2-core
   fontconfig
   freetype2
   ttf-dejavu
   ttf-liberation
   pipewire
   pipewire-pulse
   wireplumber
   polkit
   xorg-xwayland
   labwc
   foot
   wl-clipboard
   ```
   Add a separate `packaging/deps/arch-build.txt` with `base-devel pkgconf rust`
   (needed only when building from source).
2. `packaging/deps/ubuntu.txt` — the Debian/Ubuntu equivalents. **CONFIRM AT
   RUNTIME:** `apt-get install` will error on a wrong name; fix any that fail and
   record the correction. Best-effort mapping:
   ```text
   libwayland-dev
   wayland-protocols
   libxkbcommon-dev
   libinput-dev
   seatd
   libseat-dev
   libdrm-dev
   libgbm-dev
   libegl-dev
   libgles2-mesa-dev
   mesa-vulkan-drivers
   libvulkan1
   libvulkan-dev
   vulkan-tools
   libdisplay-info-dev
   libpixman-1-dev
   dbus
   at-spi2-core
   libfontconfig-1-dev
   libfreetype-dev
   fonts-dejavu
   fonts-liberation
   pipewire
   pipewire-pulse
   wireplumber
   policykit-1
   xwayland
   labwc
   foot
   wl-clipboard
   ```
   Add `packaging/deps/ubuntu-build.txt` with `build-essential pkg-config curl`
   (Rust via rustup — see Task 4.2, since Ubuntu's `rustc` is often too old).

Acceptance:
```bash
test -s packaging/deps/arch.txt && test -s packaging/deps/ubuntu.txt && \
wc -l packaging/deps/*.txt
```
→ expect: both files non-empty, line counts printed.

DO NOT:
- Duplicate this list anywhere else — installer/PKGBUILD/.deb/ISO all read these
  files.
- Invent Ubuntu names with false confidence — mark uncertain ones and confirm on
  the Ubuntu VM.

Commit: `feat(packaging): canonical Arch + Ubuntu dependency manifests`

---

### Task 4.2 — Layered `install.sh` (the primary delivery path)   [UNVERIFIED]

Precondition:
```bash
test -s packaging/deps/arch.txt && echo ok   # Task 4.1 → ok
```

Files: Create `install.sh` (repo root) — the layered installer.

Signature (CLI):
```text
sudo ./install.sh [--prefix /usr/local] [--no-deps] [--no-build] [--with-greeter] [--distro auto|arch|ubuntu]
```

Steps:
1. `set -euo pipefail`. Detect distro from `/etc/os-release` (`ID`/`ID_LIKE`)
   unless `--distro` forces it.
2. Unless `--no-deps`: install runtime deps with the native manager —
   Arch: `pacman -S --needed --noconfirm $(grep -v '^#' packaging/deps/arch.txt)`;
   Ubuntu: `apt-get update && apt-get install -y $(grep -v '^#' packaging/deps/ubuntu.txt)`.
   Install build deps too when building (see 3).
3. Unless `--no-build`: ensure a Rust toolchain (Arch: `pacman -S --needed rust`;
   Ubuntu: install via `rustup` if `cargo` absent), then
   `cargo build --release --workspace`.
4. Install the 7 binaries + `start-slopos-i` to `$PREFIX/bin` with
   `install -Dm755` (mirror `arch-install.sh:125-130`, but to `$PREFIX/bin`).
5. Run `scripts/install-session-files.sh --prefix "$PREFIX"` to place the
   session/xsession/service files. (Reuse it — do not re-implement.)
6. If `--with-greeter`: install `greetd tuigreet`, write `/etc/greetd/config.toml`
   launching `tuigreet --time --cmd start-slopos-i` (or listing
   wayland-sessions), and `systemctl enable greetd`. Print a clear message that a
   reboot / DM restart is needed to see the session.
7. Print a final summary: what was installed, where, and how to select SLOPOS-I.

Acceptance (host dry-run of the parsing/lists; full run happens on the VMs):
```bash
bash -n install.sh && echo SYNTAX-OK
grep -q 'install-session-files.sh' install.sh && grep -q 'os-release' install.sh && echo WIRED-OK
```
→ expect: `SYNTAX-OK` and `WIRED-OK`.

DO NOT:
- Partition disks or touch bootloaders — this layers onto a running system.
- Duplicate the session-file contents — call `install-session-files.sh`.
- Hardcode `/usr/local` — honor `--prefix`.

Commit: `feat(dist): layered install.sh for Arch and Ubuntu (primary path)`

---

### Task 4.3 — Arch AUR `PKGBUILD`   [UNVERIFIED]

Precondition:
```bash
test -s packaging/deps/arch.txt && echo ok
```

Files: Create `packaging/arch/PKGBUILD`.

Steps:
1. `pkgname=slopos-i`, `pkgver` from the workspace version, `arch=('x86_64')`,
   `depends=(...)` populated from `packaging/deps/arch.txt`,
   `makedepends=(cargo pkgconf)`.
2. `build()`: `cargo build --release --workspace --locked`.
3. `package()`: `install -Dm755` each of the 7 binaries + `start-slopos-i` into
   `$pkgdir/usr/bin`; install the session/xsession/service files into
   `$pkgdir/usr/share/wayland-sessions`, `.../xsessions`, `.../lib/systemd/user`
   (mirror `install-session-files.sh` targets, rooted at `$pkgdir/usr`).

Acceptance:
```bash
grep -q '^pkgname=slopos-i' packaging/arch/PKGBUILD && \
grep -q 'cargo build --release --workspace' packaging/arch/PKGBUILD && echo PKGBUILD-OK
# Full build (on the Arch VM): cd packaging/arch && makepkg -si --noconfirm
```
→ expect: `PKGBUILD-OK`; the real `makepkg` run is proven on the Arch VM (Task 4.5).

DO NOT:
- Re-list dependencies by hand — derive `depends` from `packaging/deps/arch.txt`.

Commit: `feat(packaging): AUR PKGBUILD for SLOPOS-I`

---

### Task 4.4 — Ubuntu `.deb` packaging   [UNVERIFIED]

Precondition:
```bash
test -s packaging/deps/ubuntu.txt && echo ok
```

Files: Create `packaging/debian/{control,rules,changelog,install}` (and
`compat`/`source/format` as needed for `debhelper`).

Steps:
1. `debian/control`: `Package: slopos-i`, `Depends:` populated from
   `packaging/deps/ubuntu.txt` (comma-separated), `Build-Depends: debhelper,
   cargo | rustc, pkg-config, ...` (build deps from `ubuntu-build.txt`).
2. `debian/rules`: build with `cargo build --release --workspace`; install the 7
   binaries + `start-slopos-i` to `/usr/bin` and the session files to their FHS
   paths (via `debian/install` entries).
3. `debian/changelog`: one entry at the workspace version.

Acceptance:
```bash
grep -q '^Package: slopos-i' packaging/debian/control && \
grep -q 'cargo build --release --workspace' packaging/debian/rules && echo DEB-OK
# Full build (on the Ubuntu VM): dpkg-buildpackage -us -uc -b
```
→ expect: `DEB-OK`; the real `dpkg-buildpackage` run is proven on the Ubuntu VM
(Task 4.6, optional path — `install.sh` alone satisfies the DoD).

DO NOT:
- Re-list Depends by hand — derive from `packaging/deps/ubuntu.txt`.

Commit: `feat(packaging): Debian/Ubuntu .deb packaging`

---

### Task 4.5 — VM DoD 1a: layered install on a clean **Arch** VM   [UNVERIFIED]

Precondition: a **fresh** Arch VM (VirtualBox, VMSVGA+3D for `vmwgfx` KMS — see
[SLOPOS-I.md](../SLOPOS-I.md) §3), NOT the Stage-0/1 dev VM. SSH reachable.

Steps:
1. Copy the repo in (`git clone` or `rsync`), then:
   ```bash
   sudo ./install.sh --with-greeter
   ```
2. Reboot. At the greeter, select **SLOPOS-I** and log in.
3. Confirm the desktop appears (menu bar/dock or Finder), captured as a screenshot
   to `docs/screenshots/stage4-arch-desktop.png`.

Acceptance:
```bash
# over SSH after install, before reboot:
test -f /usr/local/share/wayland-sessions/slopos-i.desktop && \
command -v slopos-compositor && command -v slopos-shell && echo ARCH-LAYERED-OK
ls -l docs/screenshots/stage4-arch-desktop.png && file docs/screenshots/stage4-arch-desktop.png
```
→ expect: `ARCH-LAYERED-OK`, a real PNG, and a **visual** confirmation that
selecting SLOPOS-I at the greeter reaches the desktop. This is **DoD part 1a**.
Record in `docs/qa/stage-4.md`.

DO NOT:
- Run this on the dev VM — the DoD is a *clean* system layering.
- Claim "reaches the desktop" from logs — the screenshot is the evidence.

Commit: `docs(qa): stage-4 DoD 1a — layered install reaches desktop on clean Arch`

---

### Task 4.6 — VM DoD 1b: layered install on a clean **Ubuntu-server** VM   [UNVERIFIED]

Precondition: a **fresh Ubuntu-server** VM (VirtualBox, VMSVGA+3D). No GUI/DM by
default — that is the point.

Steps:
1. Copy the repo in. Run:
   ```bash
   sudo ./install.sh --with-greeter --distro ubuntu
   ```
   Fix any `packaging/deps/ubuntu.txt` names that `apt-get` rejects, commit the
   correction, and re-run (idempotent).
2. Reboot. At the `greetd`/`tuigreet` prompt, launch SLOPOS-I.
3. Screenshot the desktop to `docs/screenshots/stage4-ubuntu-desktop.png`.

Acceptance:
```bash
test -f /usr/local/share/wayland-sessions/slopos-i.desktop && \
command -v slopos-compositor && echo UBUNTU-LAYERED-OK
ls -l docs/screenshots/stage4-ubuntu-desktop.png && file docs/screenshots/stage4-ubuntu-desktop.png
```
→ expect: `UBUNTU-LAYERED-OK`, a real PNG, and a **visual** desktop confirmation.
This is **DoD part 1b**. Record in `docs/qa/stage-4.md`, including every Ubuntu
package-name correction you had to make.

DO NOT:
- Skip the dep-name confirmation — Ubuntu names differ from Arch and from each
  Ubuntu release. Record what actually worked.

Commit: `docs(qa): stage-4 DoD 1b — layered install reaches desktop on clean Ubuntu-server`

---

### Task 4.7 — archiso ISO profile (secondary path)   [UNVERIFIED]

Precondition:
```bash
test -s packaging/deps/arch.txt && echo ok
```

Files: Create `packaging/iso/` — an archiso profile (copy from
`/usr/share/archiso/configs/releng/` and customize).

Steps:
1. `packaging/iso/packages.x86_64`: base live packages **plus** the contents of
   `packaging/deps/arch.txt` and `arch-build.txt` (so the image can build/run
   SLOPOS-I). Add `greetd tuigreet`.
2. `packaging/iso/airootfs/`: overlay that installs SLOPOS-I into the live root
   — either prebuild binaries into `/usr/local/bin` via a build hook, or ship the
   source and a first-boot build. Place the session files (reuse
   `install-session-files.sh` in a `customize_airootfs.sh` hook) and a greetd
   config that autostarts SLOPOS-I.
3. A build script `packaging/iso/build-iso.sh` wrapping `mkarchiso -v .`.

Acceptance:
```bash
test -f packaging/iso/packages.x86_64 && test -f packaging/iso/build-iso.sh && \
grep -qFf packaging/deps/arch.txt packaging/iso/packages.x86_64 && echo ISO-PROFILE-OK
# Full build (on an Arch host/VM with archiso): sudo bash packaging/iso/build-iso.sh
```
→ expect: `ISO-PROFILE-OK`. The actual ISO build produces `slopos-i-*.iso`,
verified by booting it (Task 4.8).

DO NOT:
- Re-declare the dependency list — the profile must include
  `packaging/deps/arch.txt` so it cannot drift from the layered path.

Commit: `feat(packaging): archiso profile for a bootable SLOPOS-I ISO`

---

### Task 4.8 — VM DoD 2: the ISO boots into SLOPOS-I   [UNVERIFIED]

Precondition: `slopos-i-*.iso` built from Task 4.7 on an Arch host/VM.

Steps:
1. Create a fresh VirtualBox VM (VMSVGA+3D, no disk needed for a live boot),
   attach the ISO, boot it.
2. Confirm it boots to the SLOPOS-I desktop (greeter auto-launch or direct).
3. Screenshot to `docs/screenshots/stage4-iso-boot.png`.

Acceptance:
```bash
ls -l docs/screenshots/stage4-iso-boot.png && file docs/screenshots/stage4-iso-boot.png
```
→ expect: a real PNG showing SLOPOS-I running from the booted ISO. This is
**DoD part 2**. Record in `docs/qa/stage-4.md` and mark Stage 4 VERIFIED once
1a + 1b + 2 all have evidence.

DO NOT:
- Mark Stage 4 VERIFIED on the ISO alone — the DoD requires the layered install on
  **both** clean Arch and clean Ubuntu-server *and* the ISO boot.

Commit: `docs(qa): stage-4 DoD 2 — bootable ISO reaches SLOPOS-I desktop`
