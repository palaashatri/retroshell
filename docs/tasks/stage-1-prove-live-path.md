# Stage 1 — Prove the Live Path (verification-first)

> **For executors:** read [docs/tasks/README.md](README.md) first. Do tasks in
> order. **Stage status: VERIFIED — DoD (a)** (2026-07-30, Windows+VBox/`vmwgfx`).
> Evidence in [docs/qa/stage-1.md](../qa/stage-1.md) and
> [docs/screenshots/stage1-finder.png](../screenshots/stage1-finder.png).

**Goal:** answer one question with evidence — *does `slopos-compositor` actually
paint a client window on real KMS?* (Originally written for virtio-gpu; this run
used VirtualBox **`vmwgfx`**.) Per the QA report the compositor never ran on real
KMS, so we do not assume; we observe.

**Why verification-first (not "fix" tasks):** QA defects C (present-buffer leak),
D (discarded libinput events), and #3 (missing frame callbacks) were **already
fixed** in commit `868b9c5` (see spec §2.1). Writing tasks to "fix" them would be
fabricated work. The one code comment that flags a *real* remaining gap is at
`crates/slopos-compositor/src/session_drm.rs:894`: "the DRM path does not yet
composite client buffers to scanout." **Observed on vmwgfx:** that comment is
outdated — the GL `DrmCompositor` path ran and painted `foot` + Finder. Stage 1
found out.

**Definition of done (from SLOPOS-I.md):** either
(a) a screenshot **captured on the VM** of Finder rendered by `slopos-compositor`
(not labwc), proving the path works end to end; or
(b) an evidenced diagnosis that isolates exactly why it does not paint —
sufficient to write the Stage-2 compositing spec.

## Global constraints

- All commands run **in the VM** over SSH (`ssh -i packaging/vm/qa_key -p 2222
  retro@127.0.0.1 '<cmd>'`) unless a task says "on the host."
- Screenshots are captured **inside the VM** (`grim`, installed in Stage 0), then
  copied to the host with `scp`. A host screenshot of the VirtualBox/UTM window is
  a fallback, not the primary evidence. (**This run:** `grim` failed — no
  screencopy protocol — so VBox `screenshotpng` was used.)
- Backend selection env vars (from `main.rs` `linux::run`): `SLOPOS_PREFER_DRM`
  (defaults on when `/dev/dri` exists), `SLOPOS_FORCE_LABWC`,
  `SLOPOS_COMPOSITOR=labwc`. To test our own compositor on KMS we must **not**
  force labwc.

---

### Task 1.1 — Capture a baseline compositor bring-up log on KMS   [VERIFIED]

Precondition:
```bash
ssh -i packaging/vm/qa_key -p 2222 retro@127.0.0.1 \
  'ls ~/slopos-i/target/release/slopos-compositor && ls /dev/dri/card0'   # → both exist
```

Steps:
1. From a **TTY** (not an SSH pty — DRM master needs a seat), run the compositor
   with logging. Do this by writing a small launch script and running it on tty1.
   Over SSH, use `systemd-run` to attach to the seat, or run via the autologin
   TTY. The portable approach: create the log-capturing invocation and trigger it
   on tty1.
   ```bash
   ssh -i packaging/vm/qa_key -p 2222 retro@127.0.0.1 'cat > ~/run-compositor.sh' <<'EOF'
   #!/usr/bin/env bash
   set -x
   export RUST_LOG=debug
   export RUST_BACKTRACE=1
   # Do NOT force labwc — we want our own DRM backend.
   unset SLOPOS_FORCE_LABWC
   unset SLOPOS_COMPOSITOR
   cd ~/slopos-i
   exec ./target/release/slopos-compositor > ~/compositor.log 2>&1
   EOF
   ssh -i packaging/vm/qa_key -p 2222 retro@127.0.0.1 'chmod +x ~/run-compositor.sh'
   ```
2. On the VM's tty1 (via the UTM console window, since it needs the seat), run
   `~/run-compositor.sh`. Let it run ~10 seconds, then switch back and stop it
   (Ctrl-C on the console, or `pkill slopos-compositor` over SSH).
3. Pull the log to the host:
   ```bash
   scp -i packaging/vm/qa_key -P 2222 retro@127.0.0.1:~/compositor.log ./stage1-bringup.log
   ```

Acceptance:
```bash
grep -iE 'drm|kms|backend|card0|scanout|error|panic' stage1-bringup.log | head -40
```
→ expect: log lines showing which backend initialized (DRM vs X11/labwc fallback)
and whether it reached the event loop. Record the full log in `docs/qa/stage-1.md`.
This task **passes** when you have a real log to read — pass/fail of the compositor
itself is decided in later tasks.

**Evidence (2026-07-30):** SSH `setsid` launch produced `session_mode=session_drm`,
EGL/GBM + GLES on SVGA3D, `DrmCompositor` active, `WAYLAND_DISPLAY=wayland-1`.
See [qa/stage-1.md](../qa/stage-1.md).

DO NOT:
- Run the compositor over a plain SSH session expecting DRM master — it needs a
  seat/TTY. Use the console for the actual run.
- Set `SLOPOS_FORCE_LABWC` — that would test labwc, not our compositor.

Commit: _none (evidence goes in qa/stage-1.md)._

---

### Task 1.2 — Determine which backend actually ran   [VERIFIED — DRM]

Precondition:
```bash
test -f stage1-bringup.log && echo ok   # → ok (from Task 1.1)
```

Steps:
1. Identify the backend from the log. The code paths to distinguish (from
   `main.rs` `linux::run` and `lib.rs` `select_backend_kind`):
   - **DRM/KMS** (what we want): messages about opening `/dev/dri/card0`, creating
     a DRM device/surface, session/libseat.
   - **X11 nested**: only under an existing X server (not our case on a bare TTY).
   - **labwc fallback**: the compositor spawned labwc instead of driving KMS
     itself.
2. Write the determination (one of: `DRM`, `X11`, `labwc-fallback`, `failed`) with
   the exact log lines that prove it into `docs/qa/stage-1.md`.

Acceptance:
```bash
grep -iE 'initializing (drm|x11)|labwc|udev|libseat|/dev/dri' stage1-bringup.log
```
→ expect: lines that unambiguously identify the backend. The QA doc entry must
name the backend and quote the proving lines.

DO NOT:
- Guess the backend from the binary name. Read the log.

Commit: _none (evidence only)._

---

### Task 1.3 — Launch a client and observe compositing   [VERIFIED — foot painted]

This is the crux: with the compositor up on DRM, does a client window paint?

Precondition:
```bash
grep -qi 'drm' stage1-bringup.log && echo "drm-backend-confirmed"   # → drm-backend-confirmed
```
If instead the log showed `labwc-fallback` or `failed`, SKIP to Task 1.5 (diagnosis)
— there is no DRM compositing to observe yet, and that itself is the finding.

Steps:
1. Extend the launch script to start Finder (or the simplest client, `foot`, as a
   sanity check) once the compositor is up. Set `WAYLAND_DISPLAY` for the client.
   ```bash
   ssh -i packaging/vm/qa_key -p 2222 retro@127.0.0.1 'cat > ~/run-with-client.sh' <<'EOF'
   #!/usr/bin/env bash
   set -x
   export RUST_LOG=info
   cd ~/slopos-i
   ./target/release/slopos-compositor > ~/compositor.log 2>&1 &
   COMP=$!
   sleep 3
   export WAYLAND_DISPLAY=$(ls "${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"/wayland-* 2>/dev/null | head -1 | xargs -n1 basename)
   echo "WAYLAND_DISPLAY=$WAYLAND_DISPLAY" >> ~/compositor.log
   # Sanity client first: foot (a known-good Wayland terminal).
   foot >> ~/client.log 2>&1 &
   sleep 5
   grim ~/screen.png || echo "grim failed" >> ~/client.log
   sleep 2
   kill $COMP 2>/dev/null
   EOF
   ssh -i packaging/vm/qa_key -p 2222 retro@127.0.0.1 'chmod +x ~/run-with-client.sh'
   ```
2. Run `~/run-with-client.sh` on the VM's tty1 console (seat required).
3. Pull the screenshot and logs:
   ```bash
   scp -i packaging/vm/qa_key -P 2222 retro@127.0.0.1:~/screen.png ./stage1-screen.png
   scp -i packaging/vm/qa_key -P 2222 retro@127.0.0.1:~/client.log ./stage1-client.log
   scp -i packaging/vm/qa_key -P 2222 retro@127.0.0.1:~/compositor.log ./stage1-with-client.log
   ```
4. Open `stage1-screen.png` on the host and look: is there a `foot` window painted,
   or a blank/black screen?

Acceptance:
```bash
ls -l stage1-screen.png && file stage1-screen.png   # → a PNG of nonzero size
```
→ expect: a real PNG. Then **visually inspect it** and record in
`docs/qa/stage-1.md` one of: "client window painted" (→ path works) or
"blank/black scanout, client alive in logs" (→ confirms the scanout gap at
session_drm.rs:894).

DO NOT:
- Declare success from logs alone — the whole point is the pixels. Look at the PNG.
- Skip the `foot` sanity check and jump straight to Finder — isolate compositor
  behavior from shell/app behavior first.

Commit: _none (evidence goes in qa/stage-1.md; screenshot is gitignored unless in docs/screenshots/)._

---

### Task 1.4 — If the client painted: capture Finder as the DoD screenshot   [VERIFIED — DoD (a)]

Precondition: Task 1.3 recorded "client window painted."

Steps:
1. Replace `foot` with Finder in `~/run-with-client.sh`:
   ```bash
   ssh -i packaging/vm/qa_key -p 2222 retro@127.0.0.1 \
     "sed -i 's#foot >> ~/client.log#~/slopos-i/target/release/slopos-shell >> ~/client.log#' ~/run-with-client.sh"
   ```
   (Finder is part of the shell; confirm the exact binary/subcommand from
   `apps/finder` and `crates/slopos-shell` if `slopos-shell` alone does not open it.)
2. Run on tty1, capture `grim ~/screen.png`, pull it to the host as
   `docs/screenshots/stage1-finder.png` (this path is NOT gitignored).
3. Visually confirm Finder rendered by our compositor.

Acceptance:
```bash
ls -l docs/screenshots/stage1-finder.png && file docs/screenshots/stage1-finder.png
```
→ expect: a PNG showing Finder. This is the **Stage 1 DoD (a)**. Mark Stage 1
VERIFIED in the QA doc and commit the screenshot.

DO NOT:
- Photoshop, upscale, or stage the screenshot. It must be `grim` output from the VM.

Commit: `docs(qa): stage-1 DoD — Finder painted by slopos-compositor on KMS`

---

### Task 1.5 — If it did NOT paint: write the evidenced diagnosis (DoD b)   [N/A — DoD (a) met]

Precondition: Task 1.2 or 1.3 recorded a failure/blank result.

Steps:
1. Assemble the diagnosis in `docs/qa/stage-1.md` from the evidence already
   captured, answering precisely:
   - Which backend ran (Task 1.2)?
   - Did the compositor accept the client connection? (grep the compositor log for
     the client's `wl_registry`/`xdg_surface` bind; the client stays alive per the
     comment at session_drm.rs:894.)
   - Was scanout blank because of the known gap (blank framebuffer presented,
     clients kept alive) or a different failure (DRM error, GL init failure on
     virtio-gpu, panic)? Quote the exact lines.
   - Is the GL `DrmCompositor` path (gated behind `composition_active`) reached, or
     is the blank-buffer path the one that runs? Quote the log/flag evidence.
2. State the single next fix the evidence points to (e.g., "wire client buffers
   into the DRM scanout via `DrmCompositor` on virtio-gpu"). This becomes the
   Stage-2 compositing spec's problem statement.

Acceptance:
```bash
grep -c 'session_drm.rs:894\|DrmCompositor\|composition_active\|blank\|scanout' docs/qa/stage-1.md
# → 3 or more (the diagnosis cites the specific code path)
```
→ expect: a diagnosis grounded in real log lines and specific code locations —
**Stage 1 DoD (b)**. Mark Stage 1 VERIFIED (the *verification* succeeded even
though the compositor path did not) and note that Stage 2 begins with the
compositing spec.

DO NOT:
- Write "it doesn't work" without the exact backend, client-bind status, and
  scanout-path evidence. A vague diagnosis is not a DoD.
- Start writing the fix here — Stage 1 ends at diagnosis; the fix is Stage 2, which
  gets its own spec→plan cycle grounded in this evidence.

Commit: `docs(qa): stage-1 diagnosis — why slopos-compositor does not paint on KMS`
