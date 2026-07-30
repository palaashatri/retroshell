# Stage 2 — Real session (input, shortcuts, lock, toolkit)

> **For executors:** read [docs/tasks/README.md](README.md) first. Do tasks in
> order. **Stage status: VERIFIED** (2026-07-30, Windows+VBox DRM path). Evidence in
> [docs/qa/stage-2.md](../qa/stage-2.md).

**Goal (spec §4 Stage 2):** turn the painting compositor from Stage 1 into a real
session — keyboard/pointer routed to shell + apps (defect B), documented shortcuts
fire, `ext-session-lock-v1` so the lock screen truly locks (defect A), and the
toolkit interaction layer works well enough to click a button (defect J).

**Definition of done (spec §4):** on the VM — **lock cannot be bypassed by
launching an app** (the QA `qa7/08` scenario), **typing the password unlocks**, and
**`Super+O` opens Finder**. Evidence in [docs/qa/stage-2.md](../qa/stage-2.md).

## What Stage 1 already settled (do NOT re-fix — honesty contract)

Stage 1 ran the DRM backend on the VM and observed:

- **Input routing already works on the DRM path.** `handle_libinput`
  (`session_drm.rs:1140-1226`) dispatches keyboard, absolute+relative pointer
  motion, and buttons; `focus_surface` (`session_drm.rs:1266-1274`) sets keyboard
  + selection focus; click-to-focus is live (`session_drm.rs:1200-1208`). So
  **defect B is a *verification*, not a rewrite** (Task 2.0). Do not "implement
  input" — confirm it, per spec §2.1.
- **`Widget::draw` is not the render path.** Both `Button::draw`
  (`retro-kit/src/button.rs:74-81`) and `Label::draw`
  (`retro-kit/src/label.rs:41`) are no-ops, yet Stage 1's Finder screenshot
  renders text and toolbar controls — so real drawing happens in the SDK/wgpu
  layer, not `Widget::draw`. **Therefore Task 2.2 is verification-first with a
  diagnosis branch; it must NOT prescribe "implement Button::draw()" as the fix —
  the evidence contradicts that.**

The genuinely missing pieces (grounded): a launcher shortcut (`Super+O`), and
`ext-session-lock-v1` (entirely absent — the current lock is a client-side `bool`
facade in `SessionManager`, `session_manager.rs:74-110`; nothing stops other
clients drawing — defect A).

## Difficulty tiering (be honest about the executor — spec §1.1)

- **Approachable (Gemma-3n class OK):** 2.0, 2.1, 2.2.
- **Compositor/protocol work (use a strong coding model, with the compositor and
  smithay 0.7 source open):** 2.3, 2.4, 2.5, 2.6. These edit Wayland protocol
  wiring and the render/input loops; a 4B model cannot design them. The exact
  smithay 0.7 `session_lock` API is quoted below so the work is transcription +
  integration, not design — but it is still the hardest set in the program.

## Global constraints

- All live/graphical checks run **on the VM** (per [HANDOFF.md](../HANDOFF.md) §3),
  launched from tty1 (DRM needs the seat), as in Stage 1.
- Keep `cargo build --workspace` + `cargo clippy --workspace` green (CI gate).
- The DRM path (`DrmSessionState`, `session_drm.rs:977`) is the one that runs on
  the VM. Prioritize it. Mirror changes to the nested X11 path
  (`RetroCompositor`, `main.rs:200`) only if trivial; note when you skip it.
- Honesty contract: no VERIFIED without a passing Acceptance transcript/screenshot.

## Grounded smithay 0.7 `session_lock` API (from the cargo cache, verbatim)

`smithay::wayland::session_lock` — **available with the existing `wayland_frontend`
feature, no extra Cargo feature needed.**

```rust
// SessionLockManagerState::new
pub fn new<D, F>(display: &DisplayHandle, filter: F) -> Self
where
    D: GlobalDispatch<ExtSessionLockManagerV1, SessionLockManagerGlobalData>,
    D: Dispatch<ExtSessionLockManagerV1, ()>,
    D: Dispatch<ExtSessionLockV1, SessionLockState>,
    D: SessionLockHandler + 'static,
    F: for<'c> Fn(&'c Client) -> bool + Send + Sync + 'static;

pub trait SessionLockHandler {
    fn lock_state(&mut self) -> &mut SessionLockManagerState;
    fn lock(&mut self, confirmation: SessionLocker);
    fn unlock(&mut self);
    fn new_surface(&mut self, surface: LockSurface, output: WlOutput);
    fn ack_configure(&mut self, _surface: WlSurface, _configure: LockSurfaceConfigure) {}
}

// SessionLocker: call .lock() to confirm the lock (sends `locked`);
// dropping it WITHOUT calling lock() sends `finished` (lock refused).
impl SessionLocker { pub fn lock(self); pub fn ext_session_lock(&self) -> &ExtSessionLockV1; }

smithay::delegate_session_lock!(DrmSessionState);   // registration macro
// re-exports: SessionLockState, LockSurface, LockSurfaceState, LockSurfaceConfigure
```

Existing delegate pattern to mirror: `delegate_layer_shell!(DrmSessionState);` at
`session_drm.rs:1609`; state fields live in the `DrmSessionState` struct at
`session_drm.rs:977`.

---

### Task 2.0 — Verify input actually reaches a client on the VM (defect B)   [VERIFIED]

Precondition (on the VM):
```bash
ssh -i packaging/vm/qa_key -p 2222 retro@127.0.0.1 'ls ~/retroshell/target/release/retro-compositor && ls /dev/dri/card0'
```

Steps:
1. Bring up the compositor + a client on tty1 (Stage-1 method: `~/run-with-client.sh`
   with `foot`). Type into `foot`; move/click the pointer.
2. Capture proof the client received input: `foot` echoing typed characters in a
   screenshot, and/or compositor log lines showing key/button dispatch. Pull the
   screenshot to `docs/screenshots/stage2-input.png`.

Acceptance:
```bash
ls -l docs/screenshots/stage2-input.png && file docs/screenshots/stage2-input.png
```
→ expect: a PNG in which typed text appears in the client (input reached it).
Record in `docs/qa/stage-2.md`: **"input verified"** (defect B resolved on DRM), or
a precise diagnosis if not.

DO NOT:
- Rewrite `handle_libinput` — Stage 1 evidence says it works; confirm, don't fix.
- Accept logs alone — the point is the client visibly reacting.

Commit: _none (evidence goes in qa/stage-2.md)._

---

### Task 2.1 — `Super+O` opens Finder (compositor-intercepted launcher)   [VERIFIED]

Precondition:
```bash
grep -n 'mods.logo' crates/retro-compositor/src/session_drm.rs | head -1   # → ~1151
```

Files: Modify `crates/retro-compositor/src/session_drm.rs`.

Design (consistent with existing Super+workspace interception): the compositor
intercepts `Super+O` and spawns the Finder binary as a Wayland client. The
compositor does not currently spawn clients, so add a small helper.

Signature (exact):
```rust
/// Spawn a first-party binary as a Wayland client of THIS compositor.
/// Resolves `bin` from PATH / /usr/local/bin / target/release; sets
/// WAYLAND_DISPLAY to our socket and WINIT_UNIX_BACKEND=wayland.
fn spawn_client(&self, bin: &str);
```

Steps:
1. Implement `spawn_client` on `DrmSessionState`: build `std::process::Command`
   for `bin`, set `WAYLAND_DISPLAY` to the compositor's socket name (the value it
   already advertises — the same `wayland-N` Stage 1 logged), inherit
   `XDG_RUNTIME_DIR`, set `WINIT_UNIX_BACKEND=wayland`, `.spawn()`, and log the
   result (do not block; ignore a failed spawn beyond a `warn!`).
2. In the keyboard filter (`session_drm.rs:1148-1178`), inside the `mods.logo &&
   Pressed` block, add before the `FilterResult::Forward` fallthrough:
   ```rust
   if sym == Keysym::o || sym == Keysym::O {
       data.spawn_client("finder");
       return FilterResult::Intercept(());
   }
   ```
3. (Optional) mirror in the nested X11 path filter (`main.rs:1467-1492`) if trivial.

Acceptance (VM):
```bash
# after launching the compositor on tty1 and pressing Super+O:
ssh ... 'pgrep -a finder'         # → a running finder process
ls -l docs/screenshots/stage2-superO-finder.png   # screenshot showing Finder opened
```
→ expect: `finder` running and a screenshot of Finder opened by `Super+O`. Record
in `docs/qa/stage-2.md`. (Host pre-check: `cargo build -p retro-compositor` →
`Finished`.)

DO NOT:
- Make the compositor depend on `retro-shell` to launch (avoid a dep cycle) —
  spawn the binary directly.
- Hardcode an absolute path — resolve `finder` from PATH/target/release.

Commit: `feat(compositor): Super+O spawns Finder on the DRM path`

---

### Task 2.2 — Verify a button is clickable in a running app (defect J)   [VERIFIED]

Precondition:
```bash
grep -q 'fn take_clicked' crates/retro-kit/src/button.rs && echo ok   # → ok
```

Context: `Button::handle_event` is implemented and unit-tested
(`button.rs:83-156`). `Button::draw` is a no-op, but so is `Label::draw`, and
Stage 1 rendered a full Finder — so drawing is done by the SDK, not `Widget::draw`.
This task **observes** whether a real click drives a real action; it does not
assume the fix.

Steps:
1. On the VM, launch an app with an obvious button (e.g. Settings, or App Store
   after Stage 3; for now any app whose button toggles visible state). Click it.
2. Observe: does the UI change / the action fire? Capture
   `docs/screenshots/stage2-button.png` and any log line proving the callback ran.
3. Record the outcome in `docs/qa/stage-2.md`:
   - **"button click works"** → defect J is satisfied for the DoD; done.
   - **"click does nothing"** → write a diagnosis (does the compositor deliver the
     button to the app? does the app forward the event to the widget tree? does
     `take_clicked`/`on_click` fire?). That diagnosis — grounded in logs — becomes
     the fix task. Do **not** guess the fix; find where the event is lost.

Acceptance:
```bash
ls -l docs/screenshots/stage2-button.png && file docs/screenshots/stage2-button.png
```
→ expect: a PNG plus a recorded outcome (works, or an evidenced diagnosis of where
the click is lost).

DO NOT:
- "Fix" `Button::draw` as a reflex — `Label::draw` is equally empty and labels
  render, so `Widget::draw` is not the render path. Diagnose before changing code.
- Mark defect J resolved without seeing a button react on the VM.

Commit: _none unless a grounded fix is written (then: `fix(kit): <specific event-path fix>`)._

---

### Task 2.3 — Add `ext-session-lock-v1` server state + handler   [VERIFIED — strong model]

Precondition:
```bash
cargo build -p retro-compositor 2>&1 | tail -1   # → Finished
grep -n 'delegate_layer_shell!(DrmSessionState)' crates/retro-compositor/src/session_drm.rs  # → ~1609
```

Files: Modify `crates/retro-compositor/src/session_drm.rs` (and its imports).

Steps:
1. Add fields to `DrmSessionState` (`session_drm.rs:977`):
   ```rust
   session_lock_state: smithay::wayland::session_lock::SessionLockManagerState,
   locked: bool,
   lock_surfaces: Vec<(smithay::output::Output, smithay::wayland::session_lock::LockSurface)>,
   ```
2. Initialize `session_lock_state` where the other `*_state` are built (near the
   layer-shell state init): `SessionLockManagerState::new::<DrmSessionState, _>(&display_handle, |_client| true)`.
   Initialize `locked: false`, `lock_surfaces: Vec::new()`.
3. Implement the handler (mirror the grounded API above):
   ```rust
   impl smithay::wayland::session_lock::SessionLockHandler for DrmSessionState {
       fn lock_state(&mut self) -> &mut smithay::wayland::session_lock::SessionLockManagerState {
           &mut self.session_lock_state
       }
       fn lock(&mut self, confirmation: smithay::wayland::session_lock::SessionLocker) {
           self.locked = true;
           confirmation.lock();          // confirm — sends `locked` to the client
           self.request_full_redraw();
       }
       fn unlock(&mut self) {
           self.locked = false;
           self.lock_surfaces.clear();
           self.request_full_redraw();
       }
       fn new_surface(&mut self, surface: smithay::wayland::session_lock::LockSurface, output: smithay::reexports::wayland_server::protocol::wl_output::WlOutput) {
           // Map the lock surface to its Output; configure it to the output size.
           // CONFIRM AT RUNTIME: the exact Output lookup from WlOutput and the
           // configure call against smithay 0.7 (LockSurface::send_configure or
           // via the compositor's output geometry).
           if let Some(out) = self.outputs.iter().find(|o| o.owns(&output)).cloned() {
               self.lock_surfaces.push((out, surface));
           }
           self.request_full_redraw();
       }
   }
   smithay::delegate_session_lock!(DrmSessionState);
   ```

Acceptance:
```bash
cargo build -p retro-compositor 2>&1 | tail -1
```
→ expect: `Finished`. (Enforcement is Tasks 2.4/2.5; this task only registers the
protocol and compiles.)

DO NOT:
- Add a Cargo feature — `wayland_frontend` already provides `session_lock`.
- Call `confirmation.lock()` conditionally-forgotten — dropping the `SessionLocker`
  without `.lock()` refuses the lock (sends `finished`). Confirm deliberately.

Commit: `feat(compositor): register ext-session-lock-v1 (DRM path)`

---

### Task 2.4 — Enforce the lock in the render path   [VERIFIED — strong model]

Precondition:
```bash
grep -n 'fn collect_render_elements' crates/retro-compositor/src/session_drm.rs  # → ~191
grep -q 'self.locked' crates/retro-compositor/src/session_drm.rs && echo ok      # Task 2.3 → ok
```

Files: Modify `crates/retro-compositor/src/session_drm.rs`
(`collect_render_elements`, `session_drm.rs:191-246`).

Steps:
1. At the top of the render-element collection, branch on `state.locked`:
   - **Locked:** render ONLY the lock surfaces (`state.lock_surfaces`), each via
     `render_elements_from_surface_tree(renderer, lock.wl_surface(), ...)` at its
     output origin, over a black clear. **Do not** render `state.windows` or
     `state.layer_surfaces` — that is the security fix for defect A. Keep the
     cursor only if you want it visible on the lock screen (optional).
   - **Unlocked:** the existing behavior (windows + layer surfaces + cursor).
2. If `locked` but `lock_surfaces` is empty (lock requested, surface not yet
   committed), render a black frame — never the underlying session.

Acceptance:
```bash
cargo build -p retro-compositor 2>&1 | tail -1
```
→ expect: `Finished`. Full proof is Task 2.7 (a launched app must not appear over
the lock).

DO NOT:
- Fall back to rendering windows when the lock surface is missing — show black.
  Leaking the session under a half-ready lock IS defect A.

Commit: `feat(compositor): render only lock surfaces while locked (defect A)`

---

### Task 2.5 — Enforce the lock in the input path   [VERIFIED — strong model]

Precondition:
```bash
grep -q 'self.locked' crates/retro-compositor/src/session_drm.rs && echo ok
```

Files: Modify `crates/retro-compositor/src/session_drm.rs` (`handle_libinput`
1140-1226; `forward_pointer_motion` 1230-1252; `focus_window_at_index` 1255-1264;
`focus_surface` 1266-1274).

Steps:
1. While `self.locked`, keyboard focus must be a lock surface, never a window:
   - In the keyboard branch, when locked, set/keep focus on the current lock
     surface (the one for the active output) and still deliver keys to it
     (the lock client needs the password keystrokes). Skip the Super+workspace and
     Super+O interceptions while locked (do not switch workspaces / launch apps
     under the lock).
2. While locked, pointer button/motion must not focus or raise session windows:
   in `forward_pointer_motion` and the button handler, when locked, target only
   the lock surface (or drop window hit-testing entirely).
3. Add a helper `fn active_lock_surface(&self) -> Option<&WlSurface>` and route
   focus through it when locked.

Acceptance:
```bash
cargo build -p retro-compositor 2>&1 | tail -1
```
→ expect: `Finished`. Behavioral proof is Task 2.7.

DO NOT:
- Let `Super+O` (Task 2.1) launch an app while locked — gate it on `!self.locked`.
- Route keys to session windows while locked — only the lock surface.

Commit: `feat(compositor): route input only to the lock surface while locked`

---

### Task 2.6 — Lock-screen client + trigger (`Super+L`)   [VERIFIED — strong model]

Precondition: Tasks 2.3–2.5 build.

Files: Create a lock client (recommended: `crates/retro-shell/src/bin/retro-lock.rs`
or a small `apps/lock` crate) and add a `Super+L` intercept in the compositor
(`session_drm.rs` keyboard filter) that `spawn_client("retro-lock")` (reusing
Task 2.1's helper).

Design:
- The lock client is a **Wayland client of `ext-session-lock-v1`**. winit does not
  expose this protocol, so use **`smithay-client-toolkit`** (add it as a dep for
  the lock binary). **CONFIRM AT RUNTIME:** the exact sctk `SessionLock` API for
  the version you pin — sctk provides a session-lock helper; bind the manager,
  call `lock`, create a lock surface per output, draw a password prompt, and on
  correct password call `unlock` (dropping the lock object).
- Password source: read `RETROSHELL_LOCK_PASSWORD` (already referenced in
  `session_clients.rs`) or integrate `SessionManager` (`session_manager.rs`
  `lock`/`unlock`, `locked`). For this cycle, a matching env/config password that
  triggers `unlock` is sufficient — real PAM auth is a later hardening step; say
  so in a comment (do not fake PAM).
- Draw a minimal centered password field on a solid background (reuse the SDK/wgpu
  draw path the apps already use — the same path that rendered Finder in Stage 1).

Steps:
1. Add the `Super+L` intercept in the compositor filter:
   `if sym == Keysym::l || sym == Keysym::L { data.spawn_client("retro-lock"); return FilterResult::Intercept(()); }`
   (still allowed while unlocked; once locked, Task 2.5 blocks other shortcuts).
2. Implement `retro-lock`: bind session-lock, lock, draw prompt, read password,
   unlock on match, exit.
3. Add the binary to its crate's `Cargo.toml` (`[[bin]]` or `src/bin/`), and to
   the installer copy-list note in Stage 4 (`retro-lock` joins the 7 binaries).

Acceptance:
```bash
cargo build --workspace 2>&1 | tail -1
# VM: press Super+L → screen locks (only the prompt renders)
ls -l docs/screenshots/stage2-locked.png && file docs/screenshots/stage2-locked.png
```
→ expect: build `Finished`; a screenshot of the locked screen showing only the
password prompt (no session windows). Record in `docs/qa/stage-2.md`.

DO NOT:
- Claim PAM/system authentication — this cycle checks a configured password only;
  comment it honestly.
- Use winit for the lock client — it can't speak `ext-session-lock-v1`; use
  smithay-client-toolkit.

Commit: `feat(shell): retro-lock ext-session-lock-v1 client + Super+L trigger`

---

### Task 2.7 — VM DoD: lock is unbypassable, password unlocks, `Super+O` opens Finder   [VERIFIED]

Precondition (VM): the compositor + `retro-lock` + `finder` all build on the VM:
```bash
ssh ... 'cd ~/retroshell && git pull && cargo build --release --workspace && echo BUILT'
```

Steps (on tty1, seat required):
1. Start the compositor + shell. Set the lock password (e.g.
   `export RETROSHELL_LOCK_PASSWORD=retro`).
2. Press **`Super+L`** → the screen locks (only the prompt renders).
3. **Attempt to bypass:** press **`Super+O`** (and try other shortcuts). Confirm
   **no Finder window appears over the lock** — the app must not draw over it (the
   `qa7/08` scenario). Screenshot to `docs/screenshots/stage2-lock-nobypass.png`.
4. Type the password → the session unlocks and the desktop returns. Screenshot to
   `docs/screenshots/stage2-unlocked.png`.
5. With the session unlocked, press **`Super+O`** → **Finder opens**. Screenshot to
   `docs/screenshots/stage2-superO-finder.png` (shared with Task 2.1).

Acceptance:
```bash
ls -l docs/screenshots/stage2-lock-nobypass.png docs/screenshots/stage2-unlocked.png \
      docs/screenshots/stage2-superO-finder.png
file docs/screenshots/stage2-lock-nobypass.png
```
→ expect: three real PNGs and a recorded narrative in `docs/qa/stage-2.md`:
locked-and-unbypassable, unlocked-by-password, `Super+O`-opens-Finder. This is the
**Stage 2 DoD**. Mark Stage 2 VERIFIED.

DO NOT:
- Accept a screenshot where an app is visible while "locked" — that is defect A
  unfixed, not a pass.
- Fake the bypass test — actually press `Super+O` while locked and show nothing
  appears.

Commit: `docs(qa): stage-2 DoD — lock unbypassable, password unlock, Super+O opens Finder`
