# Stage 3 — `.app` bundles + app store

> **For executors:** read [docs/tasks/README.md](README.md) first. Do tasks in
> order. **Stage status: UNVERIFIED** — authored 2026-07-30 from the design spec
> (§5) and verbatim code grounding; not yet run. Do not describe any task here as
> working until its Acceptance passes and the transcript is in
> [docs/qa/stage-3.md](../qa/stage-3.md).

**Goal (spec §4 Stage 3):** define the self-contained `.app` bundle format, make
`launch_services::scan_applications()` actually read `/Applications/*.app` from
disk, rewrite the app store to install `.app` bundles (not shell out to
`pacman`/`apt`/`brew`), and package the 5 first-party apps as `.app`s.

**Definition of done (spec §4):** on the VM, the store installs a `.app`, it
appears in Finder/dock, and it launches. Evidence (transcript + screenshot) in
[docs/qa/stage-3.md](../qa/stage-3.md).

## Grounding caveat (honesty contract)

Stage 1 is verified; **Stage 2 is verified** (2026-07-30, VBox DRM path). These
tasks are grounded in the current source and spec §5. If Stage 2 launch/focus
behavior changes, re-check Tasks 3.3 and 3.10 against it. **Task 3.0 re-grounds
before any change is made.** Most tasks here are **host-testable** with
`cargo test`/`cargo build` — no VM needed until the final DoD (Task 3.10).

## Planner decisions (resolve spec §10 open questions — executor must NOT re-decide)

1. **Manifest is spec §5.2 `Info.toml`**, inside `<Name>.app/Resources/Info.toml`,
   with fields `bundle_id, name, version, entrypoint, supported_types, permissions`.
   These map 1:1 to the `AppBundle` struct (`launch_services.rs:4-12`). **Note:** the
   existing per-app `apps/*/App.toml` (e.g. `entrypoint = "Executable/finder"`,
   `file_types = [...]`) is a *different, older* manifest and is **not** the bundle
   manifest. Do not copy its field names. The bundle uses `bin/<exec>` and
   `supported_types` per spec §5.1/§5.2.
2. **Integrity, not authenticity, this cycle.** The store verifies a **SHA-256
   checksum** from the catalog entry against the downloaded archive. Cryptographic
   signing (ed25519/minisign) is a future hardening step, explicitly out of scope
   here. Say so in code comments; do not fake a signature check.
3. **Catalog is a plain JSON file** (served over HTTP or read from a local path),
   an array of `{name, bundle_id, version, url, sha256, size}`. No server infra.
4. **Bundle archive format:** a gzipped tarball of the `<Name>.app/` directory
   (`<Name>.app.tar.gz`), extracted so `<Name>.app` lands directly in the install
   dir.
5. **Install target:** per-user `~/Applications` by default (no root needed on the
   VM). System `/Applications` is supported by `scan_applications` but the store
   installs to `~/Applications` to keep the VM flow root-free.

## Global constraints

- Rust edits must keep `cargo build --workspace` and `cargo clippy --workspace`
  green (CI enforces this). Add unit tests with each logic task.
- `retro-shell` already depends on `serde` (`Cargo.toml:16`) and `toml`
  (`Cargo.toml:19`) — reuse them; do not add duplicate manifest parsers.
- Do not touch the compositor in this stage. Launch integration goes through
  `crates/retro-shell/src/session_clients.rs` and `launch_services.rs`.
- Preserve the honesty contract: no task is VERIFIED without a passing Acceptance
  transcript in the QA doc.

## Current-state anchors (verbatim from grounding, 2026-07-30)

- `AppBundle` struct: `crates/retro-shell/src/launch_services.rs:4-12`
  (`bundle_id, name, version, path, entrypoint, supported_types, permissions`).
- `scan_applications()`: `launch_services.rs:72-118` — **hardcodes 5 builtins,
  never reads disk**. Search paths set at `launch_services.rs:38`
  (`/Applications`, `/User/Applications`).
- `launch_app()`: `launch_services.rs:120-122` — returns `&AppBundle`, does not exec.
- Real spawn: `crates/retro-shell/src/session_clients.rs:214-235`
  `spawn_app_client(bundle_id)`; bundle_id→binary lookup at
  `session_clients.rs:145-154`; command build at `session_clients.rs:201-211`
  (sets `WINIT_UNIX_BACKEND=wayland`, inherits `WAYLAND_DISPLAY`/`XDG_RUNTIME_DIR`).
- App store: single file `apps/appstore/src/main.rs` (~1650 lines).
  `execute_transaction` `main.rs:463-493`; `install_async` `main.rs:496-568`;
  `package_changes_allowed` `main.rs:597-602` (env
  `RETROSHELL_APPSTORE_ALLOW_PACKAGE_CHANGES=1`); install button chain
  `main.rs:1025-1028` → `start_install_async` `main.rs:970-989` →
  `backend.install_async` `main.rs:976`. No HTTP/catalog today.
- appstore deps (`apps/appstore/Cargo.toml:10-14`): `retro-sdk`, `retro-kit`,
  `tracing`, `tracing-subscriber`. **No** `reqwest`/`tar`/`sha2`/`serde`.
- App binary names: `finder, settings, textedit, terminal, appstore` →
  `target/release/<name>`.
- Theme icons already exist: `themes/{graphite,platinum,oled-graphite,high-contrast}/icons/{finder,settings,textedit,terminal}.png`
  (no `appstore.png` — Task 3.6 handles that).

---

### Task 3.0 — Re-ground: confirm the starting point   [UNVERIFIED]

Precondition:
```bash
git rev-parse --abbrev-ref HEAD   # → docs/program-design (or your work branch)
```

Steps:
1. Confirm the stub and shell-out still exist exactly where the anchors say:
   ```bash
   sed -n '72,118p' crates/retro-shell/src/launch_services.rs   # hardcoded builtins
   grep -n 'package_changes_allowed\|execute_transaction\|install_async' apps/appstore/src/main.rs
   ```
2. If line numbers drifted, update the anchors in this doc (small edit) before
   proceeding. Do not change code yet.

Acceptance:
```bash
grep -q 'For now, register built-in apps' crates/retro-shell/src/launch_services.rs && \
grep -q 'RETROSHELL_APPSTORE_ALLOW_PACKAGE_CHANGES' apps/appstore/src/main.rs && \
echo STAGE3-BASELINE-CONFIRMED
```
→ expect: `STAGE3-BASELINE-CONFIRMED`.

DO NOT:
- Edit any code in this task. This only confirms the baseline.
- Proceed if the greps fail — the anchors are wrong; fix them first.

Commit: _none (verification only)._

---

### Task 3.1 — Add the bundle-manifest parser (`bundle.rs`)   [UNVERIFIED]

Precondition:
```bash
cargo build -p retro-shell 2>&1 | tail -1   # → Finished / no errors
```

Files: Create `crates/retro-shell/src/bundle.rs`; Modify
`crates/retro-shell/src/lib.rs` (add `pub mod bundle;`).

Signature (exact):
```rust
use std::path::{Path, PathBuf};
use serde::Deserialize;
use crate::launch_services::AppBundle;

/// Parsed `Resources/Info.toml` (spec §5.2). Field names are the manifest keys.
#[derive(Debug, Clone, Deserialize)]
pub struct InfoToml {
    pub bundle_id: String,
    pub name: String,
    pub version: String,
    pub entrypoint: String,               // path within the bundle, e.g. "bin/finder"
    #[serde(default)]
    pub supported_types: Vec<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
}

#[derive(Debug)]
pub enum BundleError {
    NotABundle(PathBuf),   // dir does not end in ".app" or has no Resources/Info.toml
    Read(PathBuf, String),
    Parse(PathBuf, String),
}

/// Load one `<Name>.app` directory into an `AppBundle`.
/// `dir` must be the `.app` directory itself. `path` on the returned bundle is
/// the absolute `.app` dir; `entrypoint` is taken verbatim from Info.toml.
pub fn load_bundle(dir: &Path) -> Result<AppBundle, BundleError>;
```

Steps:
1. Implement `load_bundle`: reject if `dir` does not end with `.app`; read
   `dir/Resources/Info.toml`; `toml::from_str::<InfoToml>`; build `AppBundle`
   with `path = dir.to_string_lossy().into_owned()` and the manifest fields.
2. Add `pub mod bundle;` to `crates/retro-shell/src/lib.rs` (near the other
   `pub mod` lines).
3. Add unit tests writing a temp `.app` and asserting the parse:
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       #[test]
       fn parses_a_minimal_bundle() {
           let tmp = std::env::temp_dir().join("rs_test_TextEdit.app");
           let res = tmp.join("Resources");
           std::fs::create_dir_all(&res).unwrap();
           std::fs::write(res.join("Info.toml"),
               "bundle_id=\"com.retro.textedit\"\nname=\"TextEdit\"\nversion=\"0.1.0\"\nentrypoint=\"bin/textedit\"\nsupported_types=[\"txt\"]\npermissions=[\"files.read\"]\n").unwrap();
           let b = load_bundle(&tmp).unwrap();
           assert_eq!(b.bundle_id, "com.retro.textedit");
           assert_eq!(b.entrypoint, "bin/textedit");
           assert_eq!(b.supported_types, vec!["txt"]);
           std::fs::remove_dir_all(&tmp).ok();
       }
       #[test]
       fn rejects_non_bundle_dir() {
           assert!(load_bundle(std::path::Path::new("/tmp")).is_err());
       }
   }
   ```

Acceptance:
```bash
cargo test -p retro-shell bundle:: 2>&1 | grep -E 'test result:' 
```
→ expect: `test result: ok.` with 2 passed.

DO NOT:
- Reuse `apps/*/App.toml` field names — the bundle manifest is `Info.toml` per
  spec §5.2 (`bin/<exec>`, `supported_types`).
- Add a new toml/serde dependency — `retro-shell` already has both.

Commit: `feat(shell): parse .app Resources/Info.toml into AppBundle`

---

### Task 3.2 — `scan_applications()` reads `.app` dirs from disk   [UNVERIFIED]

Precondition:
```bash
cargo test -p retro-shell bundle:: >/dev/null 2>&1 && echo ok   # Task 3.1 done → ok
```

Files: Modify `crates/retro-shell/src/launch_services.rs`.

Steps:
1. Add `~/Applications` to the search paths in `LaunchServices::new()`
   (`launch_services.rs:38`): push
   `format!("{}/Applications", std::env::var("HOME").unwrap_or_default())`
   after the two existing entries. Keep `/Applications` and `/User/Applications`.
2. Replace the body of `scan_applications()` (`launch_services.rs:72-118`) so it
   walks each search path, reads every child directory ending in `.app`, calls
   `crate::bundle::load_bundle`, and `register_bundle` on success. On parse
   failure, log via `tracing::warn!` and skip (do not panic). Signature stays
   `pub fn scan_applications(&mut self)`.
3. Add a unit test: create a temp dir with `Foo.app/Resources/Info.toml`, point a
   `LaunchServices` search path at it, call `scan_applications`, assert the bundle
   registered. (Construct the struct directly to inject the temp search path.)

Acceptance:
```bash
cargo test -p retro-shell 2>&1 | grep -E 'test result:' | tail -1
```
→ expect: `test result: ok.` (all retro-shell tests pass, including the new scan test).

DO NOT:
- Keep the hardcoded builtin list — it is deleted. (Bundles now come from disk;
  Task 3.6 lays the 5 first-party bundles on disk.)
- Panic or `unwrap()` on a bad/missing Info.toml — warn and skip.
- Recurse into subdirectories — only direct `*.app` children of each search path.

Commit: `feat(shell): scan_applications reads /Applications/*.app from disk`

---

### Task 3.3 — Launch resolves `path + entrypoint` and execs   [UNVERIFIED]

Precondition:
```bash
grep -q 'fn spawn_app_client' crates/retro-shell/src/session_clients.rs && echo ok   # → ok
```

Files: Modify `crates/retro-shell/src/session_clients.rs`.

Context: today `spawn_app_client` maps `bundle_id`→binary via a hardcoded table
(`session_clients.rs:145-154`) and runs it from `PATH`. Disk bundles must instead
exec `<bundle.path>/<entrypoint>`.

Steps:
1. Add a function that, given an `&AppBundle`, builds a `std::process::Command`
   for `Path::new(&bundle.path).join(&bundle.entrypoint)`, sets the same env the
   current `build_app_command` sets (`session_clients.rs:201-211`:
   `WINIT_UNIX_BACKEND=wayland`, inherit `WAYLAND_DISPLAY`/`XDG_RUNTIME_DIR`,
   remove `RETROSHELL_LOCK_PASSWORD`), and `spawn()`s it.
   Signature: `pub fn spawn_bundle(bundle: &crate::launch_services::AppBundle) -> std::io::Result<std::process::Child>`.
2. In the launch path, prefer `spawn_bundle` when the resolved `AppBundle` has a
   real on-disk `entrypoint` (i.e. `Path::new(&bundle.path).join(&bundle.entrypoint)`
   exists); fall back to the existing binary-name table only if it does not.
3. Add a unit test that builds the `Command` for a fake bundle and asserts the
   program path equals `path/entrypoint` (use a `Command`-inspection helper or
   assert on a small pure builder fn that returns the resolved `PathBuf`).

Acceptance:
```bash
cargo build -p retro-shell 2>&1 | tail -1 && cargo test -p retro-shell 2>&1 | grep -E 'test result:' | tail -1
```
→ expect: build `Finished`, tests `ok.`

DO NOT:
- Remove the fallback binary table yet — first-party dev builds still rely on it
  until Task 3.6 lays bundles on disk.
- Change the compositor or the Wayland env keys — reuse exactly what
  `build_app_command` already sets.

Commit: `feat(shell): launch execs <bundle>/<entrypoint> as a Wayland client`

---

### Task 3.4 — Bundle-builder script for one app   [UNVERIFIED]

Precondition:
```bash
ls target/release/finder 2>/dev/null || cargo build --release -p finder 2>&1 | tail -1
```

Files: Create `packaging/apps/build-app-bundle.sh` (builds ONE app into a `.app`).

Signature (CLI):
```text
build-app-bundle.sh <app-crate> <Display Name> <bundle_id> <version> <OUTDIR> [icon.png]
# e.g. build-app-bundle.sh finder "Finder" com.retro.finder 0.1.0 /tmp/Applications \
#        themes/platinum/icons/finder.png
```

Steps:
1. Script: `set -euo pipefail`. Build `cargo build --release -p "$app"`.
   Create `"$OUTDIR/$Name.app"/{Resources,bin}`. `install -m755
   target/release/$app "$OUTDIR/$Name.app/bin/$app"`. Write
   `Resources/Info.toml` in the spec §5.2 format with
   `entrypoint = "bin/$app"`. Copy the icon (if given) to
   `Resources/icon.png`; otherwise skip (Info.toml still valid).
2. `supported_types`/`permissions`: for `finder` use `[]`/`["files.read","files.write"]`;
   for `textedit` use `["txt","md","rtf"]`/`["files.read","files.write"]`; others `[]`/`[]`.
   (Encode a small case-statement; do not prompt.)

Acceptance:
```bash
bash packaging/apps/build-app-bundle.sh finder "Finder" com.retro.finder 0.1.0 /tmp/rs-apps themes/platinum/icons/finder.png
test -x /tmp/rs-apps/Finder.app/bin/finder && \
grep -q 'bundle_id = "com.retro.finder"' /tmp/rs-apps/Finder.app/Resources/Info.toml && \
grep -q 'entrypoint = "bin/finder"' /tmp/rs-apps/Finder.app/Resources/Info.toml && \
echo BUNDLE-BUILD-OK
```
→ expect: `BUNDLE-BUILD-OK`.

DO NOT:
- Hardcode a single app — the crate/name/id are arguments.
- Emit the old `App.toml` field names (`Executable/`, `file_types`). Use `Info.toml`
  per spec §5.2.

Commit: `feat(packaging): build-app-bundle.sh assembles one .app bundle`

---

### Task 3.5 — Package all 5 first-party apps   [UNVERIFIED]

Precondition:
```bash
test -f packaging/apps/build-app-bundle.sh && echo ok   # Task 3.4 → ok
```

Files: Create `packaging/apps/build-all-bundles.sh`.

Steps:
1. Call `build-app-bundle.sh` once per app into `${OUTDIR:-/tmp/Applications}`:
   - `finder "Finder" com.retro.finder` icon `themes/platinum/icons/finder.png`
   - `settings "Settings" com.retro.settings` icon `themes/platinum/icons/settings.png`
   - `textedit "TextEdit" com.retro.textedit` icon `themes/platinum/icons/textedit.png`
   - `terminal "Terminal" com.retro.terminal` icon `themes/platinum/icons/terminal.png`
   - `appstore "App Store" com.retro.appstore` (no icon — none exists; **CONFIRM
     AT RUNTIME:** if `themes/platinum/icons/appstore.png` was added later, pass it).
   Use version `0.1.0` for all.

Acceptance:
```bash
OUTDIR=/tmp/rs-apps bash packaging/apps/build-all-bundles.sh && \
ls -d /tmp/rs-apps/*.app | wc -l   # → 5
```
→ expect: `5` (Finder.app, Settings.app, TextEdit.app, Terminal.app, "App Store.app").

DO NOT:
- Fail the whole script if the appstore icon is missing — it is optional.

Commit: `feat(packaging): build-all-bundles.sh packages the 5 first-party apps`

---

### Task 3.6 — Remove the package-manager path from the store   [UNVERIFIED]

Precondition:
```bash
grep -q 'RETROSHELL_APPSTORE_ALLOW_PACKAGE_CHANGES' apps/appstore/src/main.rs && echo ok
```

Files: Modify `apps/appstore/src/main.rs`.

Steps (spec §5.3 — delete, do not re-gate):
1. Delete `execute_transaction` (`main.rs:463-493`), `install_async`
   (`main.rs:496-568`), and `package_changes_allowed` (`main.rs:597-602`).
2. Delete the `RETROSHELL_APPSTORE_ALLOW_PACKAGE_CHANGES` env var and any
   pacman/apt/brew/dnf/pkg/apk/zypper `Command` shell-outs and the
   `PackageManager`/`PackageBackend` types that only served them. Keep the UI
   scaffolding (list view, install button) — it is rewired in Task 3.8.
3. Temporarily stub the install button handler to a no-op comment
   `// rewired in Task 3.8` so the crate still builds.

Acceptance:
```bash
cargo build -p appstore 2>&1 | tail -1 && \
! grep -q 'RETROSHELL_APPSTORE_ALLOW_PACKAGE_CHANGES\|pacman\|apt-get' apps/appstore/src/main.rs && \
echo PACKAGE-PATH-REMOVED
```
→ expect: build `Finished` and `PACKAGE-PATH-REMOVED`.

DO NOT:
- Re-gate the package path — the decision (spec §5.3) is to **remove** it. Package
  managers are reached via the Terminal app, not the store.
- Delete the whole file — keep the app’s window/UI so Task 3.8 can rewire it.

Commit: `refactor(appstore): remove pacman/apt system-package path (spec §5.3)`

---

### Task 3.7 — `.app` installer core (fetch → verify → extract → move → rescan)   [UNVERIFIED]

Precondition:
```bash
cargo build -p appstore >/dev/null 2>&1 && echo ok   # Task 3.6 → ok
```

Files: Create `apps/appstore/src/bundle_install.rs`; Modify
`apps/appstore/Cargo.toml` (add deps) and `apps/appstore/src/main.rs`
(`mod bundle_install;`).

Deps to add (`apps/appstore/Cargo.toml`): `sha2 = "0.10"`, `tar = "0.4"`,
`flate2 = "1"`, `serde = { workspace = true }`, `serde_json = "1"`. (No network
client this cycle — install from a local/`file://` path or an already-downloaded
archive; a real HTTP fetch is Task 3.9's optional extension.)

Signature (exact):
```rust
use std::path::{Path, PathBuf};

#[derive(serde::Deserialize, Clone, Debug)]
pub struct CatalogEntry {
    pub name: String,
    pub bundle_id: String,
    pub version: String,
    pub url: String,       // file:// path or http(s) URL (fetch is Task 3.9)
    pub sha256: String,    // lowercase hex of the .app.tar.gz
    #[serde(default)]
    pub size: u64,
}

#[derive(Debug)]
pub enum InstallError { Io(String), Checksum{expected:String, got:String}, Extract(String), NoDotApp }

/// Verify `archive`'s sha256 == `expected` (integrity only; signing is future
/// work — spec §5.3 / Stage-3 planner decision 2). Then extract the .app.tar.gz
/// into a staging dir and atomically rename the top-level `<Name>.app` into
/// `install_dir`. Returns the installed `<Name>.app` path.
pub fn install_from_archive(archive: &Path, expected_sha256: &str, install_dir: &Path)
    -> Result<PathBuf, InstallError>;

/// Parse a JSON catalog (array of CatalogEntry) from bytes.
pub fn parse_catalog(bytes: &[u8]) -> Result<Vec<CatalogEntry>, InstallError>;
```

Steps:
1. `install_from_archive`: stream the file through `sha2::Sha256`, compare to
   `expected_sha256` (lowercase hex); on mismatch return `Checksum`. Extract the
   gzip+tar into `install_dir/.staging-<pid>/`; find the single top-level dir
   ending in `.app`; `std::fs::rename` it to `install_dir/<Name>.app` (atomic on
   same filesystem); clean the staging dir.
2. `parse_catalog`: `serde_json::from_slice`.
3. Unit tests: build a tiny `.app.tar.gz` in a temp dir, compute its sha256, call
   `install_from_archive`, assert the `.app` lands in the target and a wrong
   checksum returns `Checksum`.

Acceptance:
```bash
cargo test -p appstore bundle_install:: 2>&1 | grep -E 'test result:'
```
→ expect: `test result: ok.` (checksum-pass, checksum-fail, and parse tests).

DO NOT:
- Claim to verify a cryptographic signature — this is SHA-256 integrity only.
  Say so in a comment (planner decision 2).
- Extract with a path that could escape `install_dir` (reject tar entries with
  `..` or absolute paths — a tar traversal guard).

Commit: `feat(appstore): .app installer — sha256 verify, extract, atomic install`

---

### Task 3.8 — Wire the install button to the `.app` installer   [UNVERIFIED]

Precondition:
```bash
cargo test -p appstore bundle_install:: >/dev/null 2>&1 && echo ok   # Task 3.7 → ok
```

Files: Modify `apps/appstore/src/main.rs`.

Steps:
1. Load a catalog at startup: read `RETROSHELL_APPSTORE_CATALOG` (a path to a
   JSON file); if unset, default to `~/Applications/catalog.json`; if absent, show
   an empty list (no panic). Populate the store’s list view from
   `parse_catalog`.
2. Replace the stubbed install handler (from Task 3.6) so
   `install_button.take_clicked()` (was `main.rs:1025`) calls
   `bundle_install::install_from_archive` for the selected `CatalogEntry`
   (resolving `url` as a local/`file://` path this cycle) into
   `~/Applications`, then invokes a rescan hook (see Task 3.10).
3. Show success/failure in the existing status area.

Acceptance:
```bash
cargo build -p appstore 2>&1 | tail -1 && \
grep -q 'install_from_archive' apps/appstore/src/main.rs && echo INSTALL-WIRED
```
→ expect: build `Finished` and `INSTALL-WIRED`.

DO NOT:
- Block the UI thread on a large extract without at least a status message.
- Re-introduce any package-manager call.

Commit: `feat(appstore): install button installs a .app from the catalog`

---

### Task 3.9 — (Optional) HTTP fetch for catalog + archives   [UNVERIFIED]

Precondition: Task 3.8 done. This task is **optional** for the DoD (local/`file://`
install already satisfies it) but makes the store usable over a network.

Files: Modify `apps/appstore/Cargo.toml` (+`ureq = "2"` — a tiny blocking HTTP
client, no async runtime) and `apps/appstore/src/bundle_install.rs`.

Steps:
1. Add `fn fetch(url: &str) -> Result<Vec<u8>, InstallError>` using `ureq` for
   `http(s)://`, falling back to reading the path for `file://`/bare paths.
2. Use it to load the catalog and the archive when `url` is remote.

Acceptance:
```bash
cargo build -p appstore 2>&1 | tail -1
```
→ expect: `Finished`. (Networked install is exercised on the VM in Task 3.10 if
you choose the HTTP path.)

DO NOT:
- Pull in `tokio`/`reqwest` (async) — this store is synchronous; `ureq` is enough.

Commit: `feat(appstore): optional http(s) fetch for catalog and bundles`

---

### Task 3.10 — VM DoD: install a `.app`, see it in Finder, launch it   [UNVERIFIED]

Precondition (on the VM, per [HANDOFF.md](../HANDOFF.md) §3):
```bash
ssh -i packaging/vm/qa_key -p 2222 retro@127.0.0.1 'cd ~/retroshell && git pull && cargo build --release --workspace && echo BUILT'
```
→ expect: `BUILT`.

Steps (run over SSH to the VM unless noted):
1. Build bundles on the VM and stage a catalog:
   ```bash
   ssh ... 'cd ~/retroshell && OUTDIR=$HOME/Applications bash packaging/apps/build-all-bundles.sh'
   ```
2. Make one app installable *through the store* (not pre-placed): tar+gz one
   bundle (e.g. `TextEdit.app`) into `~/store/TextEdit.app.tar.gz`, compute its
   sha256, write `~/Applications/catalog.json` with that one entry
   (`url` = the local tarball path), and **remove** the pre-built `TextEdit.app`
   from `~/Applications` so the store is what installs it.
3. Add a **rescan hook**: after the store installs, `scan_applications` must pick
   the new bundle up. If the shell is running, confirm it re-scans on focus or on
   a signal; otherwise restart `retro-shell`. (This is the integration Task 3.2/3.3
   set up; **CONFIRM AT RUNTIME** how the running shell triggers a rescan and
   record it.)
4. Launch the compositor + shell on tty1 (Stage-1 method), open the store, install
   TextEdit, confirm it appears in Finder/dock, and launch it.
5. Capture a screenshot to `docs/screenshots/stage3-appstore-install.png` (Finder
   or dock showing the installed app) and one of the launched app.

Acceptance:
```bash
# on the VM, after the store install:
ssh ... 'test -x $HOME/Applications/TextEdit.app/bin/textedit && echo INSTALLED-VIA-STORE'
ls -l docs/screenshots/stage3-appstore-install.png && file docs/screenshots/stage3-appstore-install.png
```
→ expect: `INSTALLED-VIA-STORE`, a real PNG, and a **visual** confirmation that
the installed app shows in Finder/dock and launches. This is the **Stage 3 DoD**.
Record everything in `docs/qa/stage-3.md` and mark Stage 3 VERIFIED.

DO NOT:
- Pre-place the app you claim the store installed — the store must be what puts it
  in `~/Applications`. Pre-placing it and calling it a store install is fabrication.
- Mark VERIFIED from `cargo test` alone — the DoD is the app appearing and
  launching on the VM, in a screenshot.

Commit: `docs(qa): stage-3 DoD — store installs a .app that launches on the VM`
