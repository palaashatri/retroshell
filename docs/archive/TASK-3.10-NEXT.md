# Task 3.10 — Ready for VM Verification

**Status:** Code complete; awaiting final VM DoD run.

## What's Done (Tasks 3.0–3.8)

All implementation is in place and tested:
- ✅ Bundle parser (`crates/slopos-shell/src/bundle.rs`) — parses `Resources/Info.toml`
- ✅ Disk scanner (`scan_applications()`) — reads `*.app` directories from `~/Applications` and `/Applications`
- ✅ Launch resolver — execs `<path>/<entrypoint>` from AppBundle
- ✅ Bundle packaging scripts — `packaging/apps/build-all-bundles.sh` packages 5 first-party apps
- ✅ Appstore installer — `apps/appstore/src/bundle_install.rs` implements SHA-256 integrity, tar extraction, atomic move
- ✅ Install button wired — store UI connected to installer
- ✅ Package-manager path removed — no more pacman/apt in the store

**Evidence:** See `docs/qa/stage-3.md` for unit test transcripts and code anchors.

## What's Pending: Task 3.10 (VM DoD)

The final step is a **runtime verification on the VM**:
1. Build all `.app` bundles
2. Stage one bundle (TextEdit.app) in the store catalog
3. Launch the compositor + shell
4. Open App Store, install TextEdit
5. Verify it appears in Finder/dock and launches
6. Capture screenshots
7. Update `docs/qa/stage-3.md` with results

## How to Run Task 3.10

### On Your Host (before SSH-ing to the VM)

Ensure the code is synced to the VM:

```bash
rsync -az --exclude target --exclude .git --exclude docs/screenshots \
  -e "ssh -i ~/.ssh/slopos-i_utm" ./ ubuntu@192.168.64.15:/home/ubuntu/slopos-i/
```

### On the VM

```bash
ssh -i ~/.ssh/slopos-i_utm ubuntu@192.168.64.15

# Inside the VM:
cd ~/slopos-i

# Build the workspace (this takes ~2 min for code-only changes)
cargo build --release --workspace

# Run the Task 3.10 setup script
bash packaging/vm/task-3.10-dod.sh
```

The script will:
1. Build all 5 `.app` bundles to `~/Applications/`
2. Create a store-installable tarball of TextEdit.app
3. Generate a catalog JSON pointing to it
4. **Remove** the pre-built TextEdit.app (so the store must install it)
5. Print instructions for the manual steps

Then, in another terminal or tty on the VM:

```bash
# Set environment
export XDG_RUNTIME_DIR=/run/user/1000 \
       LIBSEAT_BACKEND=seatd \
       LIBGL_ALWAYS_SOFTWARE=1 \
       GALLIUM_DRIVER=llvmpipe \
       SLOPOS_LAYER_SHELL_CHROME=1

cd ~/slopos-i

# Run the compositor (it spawns the shell)
./target/release/slopos-compositor
```

Once the shell is running:
1. Open App Store from the menu bar or dock
2. Find TextEdit in the app list
3. Click Install
4. Wait for the install to complete
5. Verify TextEdit appears in Finder/dock
6. Launch TextEdit from Finder or dock

### Capture Screenshots

After store install and app launch:

**For Xvfb method (if running headless):**
```bash
export DISPLAY=:99
import -window root ~/slopos-i/docs/screenshots/stage3-appstore-install.png
import -window root ~/slopos-i/docs/screenshots/stage3-textedit-launched.png
```

**For VirtualBox:**
```bash
VBoxManage controlvm slopos-i-arch screenshotpng ~/slopos-i/docs/screenshots/stage3-appstore-install.png
```

### Verify Installation

After the store install completes:

```bash
# This should print INSTALLED-VIA-STORE
test -x $HOME/Applications/TextEdit.app/bin/textedit && echo INSTALLED-VIA-STORE
```

## Update the QA Doc

Once Task 3.10 completes:

1. Add the command transcript to `docs/qa/stage-3.md` (Step "Task 3.10 — DoD on Env A, 2026-07-31")
2. Reference the PNG files under `## Transcripts`
3. Update the result table: Task 3.10 → PASS
4. Update the stage header: "Stage status: VERIFIED (2026-07-31)"

Then merge back to `main`.

## Files Modified This Session

- `Cargo.lock` — updated deps (tar, sha2, flate2, serde_json)
- `packaging/vm/task-3.10-dod.sh` — helper script for VM verification (new)
- `docs/TASK-3.10-NEXT.md` — this file (new)

## Notes

- The code builds and passes all unit tests (`cargo test --workspace` on the VM).
- The UI flow (store → install → rescan → Finder) is wired; no code is broken.
- The only unknown is whether the visual flow works end-to-end on the real display — hence Task 3.10.
- No code changes are expected for Task 3.10; it is pure verification.
