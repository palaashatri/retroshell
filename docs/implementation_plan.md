# RetroShell — Execution Plan (Haiku + Subagents)

> **Purpose**: This is a work order, not a narrative. It is written to be executed by a
> cheaper model (Haiku) running several subagents in parallel. Every task is self-contained:
> exact files, exact line anchors, exact change, and an exact verify command with a pass
> condition. Do not improvise scope.
>
> **Source of truth for current state**: [`docs/audit_2026-07-09.md`](audit_2026-07-09.md)
> (fresh, evidence-based, `file:line`-backed). The old scoring narrative that used to live
> here is superseded by that audit. Historical sprint notes are preserved in git history.

---

## Rules of engagement (read first — applies to EVERY subagent)

1. **No fabrication.** Never claim a test passed, a build succeeded, or a file changed unless
   a tool call in *your* session produced that result. If you didn't run it, say "not run".
2. **Evidence or it didn't happen.** When you report a task done, paste the exact command you
   ran and its output (the `test result:` line, the `DOCKER_EXIT=` line, etc.).
3. **Read before you edit.** Open the file and confirm the line anchors below still match
   before changing anything. Line numbers drift — anchor on the quoted code, not the number.
4. **Stay in your task's files.** Do not touch files owned by another parallel task.
5. **Compositor code is Linux-only.** `crates/retro-compositor/src/main.rs` is behind
   `#[cfg(target_os = "linux")]`. It does **not** build on macOS (only the 5-line stub does).
   Any compositor change must be verified with `docker build`, not host `cargo`.
6. **Run the verify command.** A task is done only when its acceptance command prints its
   pass condition. Report the literal output.

---

## Dependency graph & parallelization

```
T0 (build fix + verify) ──┬─> T1 (compositor runtime smoke)
                          └─> T4 (wl_data_device send)      [compositor, needs Linux build]

T2 (screen lock)      ─┐
T3 (themes)           ─┤  all independent of each other and of T0 — RUN IN PARALLEL
T5 (accessibility)    ─┤
T6 (doc reconcile)    ─┘  T6 should run LAST (after T2/T3/T5 land) so docs match code
```

- **Wave 1 (parallel):** T0, T2, T3, T5
- **Wave 2 (parallel, after T0):** T1, T4
- **Wave 3 (serial, after T2/T3/T5):** T6

Assign one subagent per task. T0 and T1/T4 need Docker; T2/T3 need only host `cargo` (they
touch cross-platform crates); T5/T6 are cheap.

---

## Baseline (already true — do not redo)

- Host tests pass: `cargo test --workspace --exclude retro-compositor` → **135 passed, 0 failed** (verified 2026-07-09).
- `.dockerignore` already fixed to exclude `target/` and `target-docker/` (a 5.7 GB dir that previously broke `docker build` with "no space left on device"). Do not re-add those artifact dirs to the image.

---

## T0 — Fix the compositor compile error so the image can build at all  `[✅ DONE 2026-07-09]`

> **Status: COMPLETE.** The one-line fix in step 1 was applied and verified:
> `docker build -t retroshell .` → `DOCKER_EXIT=0`, `retro-compositor` compiles, binary present
> in the image. The Dockerfile cache-block optimization (step 2) is still worth doing but
> optional. Remaining subagents can skip straight to T1–T6.

**Why (PROVEN, not speculative):** `docker build` currently **fails** — exit 1, with
`cargo build --release --workspace` exit 101. `retro-compositor` has a type error, and because
it's a workspace member, the failure kills the build of *every* binary including `retro-shell`.
**A from-scratch image cannot be produced today.** The `target-docker/` binaries are stale
(Jul 4, before the compositor existed), which is why this was invisible.

The error (verified 2026-07-09):
```
error[E0308]: mismatched types
  --> crates/retro-compositor/src/main.rs:564:43
564|  let pos = ev.position_transformed(state.output_size);
   |    expected `Size<i32, Logical>`, found `Size<i32, Physical>`
```

**Files:** `crates/retro-compositor/src/main.rs`, then `Dockerfile`.

**Changes:**
1. **Fix the type error** in `handle_pointer_motion` (~line 564). `position_transformed` takes
   a `Size<i32, Logical>`; `state.output_size` is `Physical`. Scale is 1 under the nested-X11
   backend, so build a logical size from the same dimensions:
   ```rust
   let logical = smithay::utils::Size::<i32, Logical>::from(
       (state.output_size.w, state.output_size.h),
   );
   let pos = ev.position_transformed(logical);
   ```
   (`Logical` is already imported via `smithay::utils::{... Logical ...}` — confirm.)
   Then fix the **3 warnings** the build reported if they are trivial (unused imports).
2. **Dockerfile cache block** (optimization, so rebuilds are fast): in the manifest-copy block
   (~line 44) add `COPY crates/retro-compositor/Cargo.toml crates/retro-compositor/`; in the
   dummy-source block (~line 57) add `crates/retro-compositor` to both the `mkdir -p` list and
   the `for d in ...` loop, and add `touch crates/retro-compositor/src/lib.rs` beside the other
   `touch` lines.

**Acceptance:**
```bash
docker build -t retroshell . > /tmp/t0.log 2>&1; echo "DOCKER_EXIT=$?"
grep -c 'Compiling retro-compositor' /tmp/t0.log   # must be >= 1
docker run --rm --entrypoint ls retroshell -l /usr/local/bin/retro-compositor
```
**Pass condition:** `DOCKER_EXIT=0`, the grep prints `1`, and the `ls` shows the binary.
Paste all three outputs in your report.

---

## T1 — Compositor actually runs at runtime (not the labwc fallback)  `[Docker, needs T0]`

**Why:** `docker-entrypoint.sh:46-92` tries `retro-compositor`, then **silently falls back to
labwc** if it dies within 3s. No evidence exists that the compositor ever served `retro-shell`.

**Files:** none (verification only) unless a bug is found.

**Acceptance:**
```bash
docker run -d --name rs-smoke -p 6080:6080 retroshell
sleep 12
docker logs rs-smoke 2>&1 | grep -E 'retro-compositor is running|falling back to labwc|WAYLAND_DISPLAY='
docker exec rs-smoke sh -c 'ps aux | grep -E "[r]etro-compositor|[r]etro-shell"'
docker rm -f rs-smoke
```
**Pass condition:** logs contain `retro-compositor is running` (NOT `falling back to labwc`),
`WAYLAND_DISPLAY=` shows a socket name, and both `retro-compositor` and `retro-shell`
processes are alive. If it falls back to labwc, capture the compositor's stderr
(`docker logs`) — that error is the real bug; report it, do not paper over it.

---

## T2 — Screen lock must not unlock on any keypress  `[host cargo]`

**Why:** Documented as "PAM-backed", but there is no PAM and **any keypress unlocks**.
Evidence: `crates/retro-shell/src/lib.rs:1856-1861` unlocks unconditionally; the lock window
(`lib.rs:1475-1482`) only says "Press any key to unlock".

**Scope (keep it honest and small):** Implement a real password gate. Do **not** claim PAM
unless you actually wire libpam. Minimal correct behavior:
- The lock screen shows a password `TextField` (masked).
- Unlock succeeds **only** when the entered text equals the expected secret; wrong/empty input
  shows "Incorrect password" and stays locked. Random non-Enter keys must never unlock.
- Expected secret source, in order: env `RETROSHELL_LOCK_PASSWORD`, else a value in
  `~/.config/retroshell/settings.conf` key `lock_password`, else lock is disabled and Cmd+L
  shows a notification "Lock password not set" instead of locking.

**Files:** `crates/retro-shell/src/lib.rs` (`build_lock_screen_window`, the unlock handler
~1856-1861, and the `shell.lock` action ~827-830).

**Acceptance:**
```bash
cargo test -p retro-shell lock 2>&1 | grep 'test result:'
```
Add a unit test `lock_rejects_wrong_password` and `lock_accepts_correct_password` covering the
verify function. **Pass condition:** `test result: ok. ... 0 failed` and the two new tests are
listed. Also confirm no `Press any key to unlock` string remains: `grep -rn "any key to unlock" crates/` returns nothing.

---

## T3 — Make the theme set match its documentation  `[host cargo]`

**Why:** Docs claim themes "Classic, Dark, High Contrast, Solarized, Dracula". Actual enum is
Classic, Dark, Grape, Blueberry, Strawberry (`crates/retro-shell/src/theme_manager.rs:7-20`).

**Decision (do this, don't re-litigate):** ADD three real themes — `Solarized`, `Dracula`,
`HighContrast` — as new `ThemeName` variants. Keep Grape/Blueberry/Strawberry. Final set = 8.

**Files:** `crates/retro-shell/src/theme_manager.rs`

**Changes:** For each new variant, add it to: the `ThemeName` enum, `accent_color()`, every
other `match self { ... }` in the file (grep for `Self::Strawberry` to find them all — each
exhaustive match will fail to compile until you add the arm, which is your safety net), the
`as_str()`/`from_str` (de)serialization, and the ordered list the Settings > Appearance pane
iterates. Use standard palettes: Solarized base03 `#002b36`/accent `#268bd2`; Dracula bg
`#282a36`/accent `#bd93f9`; High Contrast pure black bg / white fg / yellow accent.

**Acceptance:**
```bash
cargo test -p retro-shell theme 2>&1 | grep 'test result:'
cargo build --workspace --exclude retro-compositor 2>&1 | tail -1
```
Add a test asserting all 8 variants round-trip through `as_str`/`from_str` and that
`from_str("Dracula")` etc. resolve. **Pass condition:** build `Finished`, `0 failed`.

---

## T5 — Accessibility: make the code and docs tell the same (true) story  `[host cargo, cheap]`

**Why:** `register_at_spi_app()` (`crates/retro-kit/src/accessibility.rs:215-238`) is an
explicit stub that silent-fails and exposes no `Accessible` object tree, yet docs imply a
registered `org.a11y.Bus` service. A full AT-SPI2 implementation is out of scope (large,
expensive). **Required:** make claims honest. **Optional/stretch:** minimal real registration.

**Files:** `crates/retro-kit/src/accessibility.rs` (comments/log text only for the required
part).

**Change (required):** Ensure the doc comment on `register_at_spi_app` and its log line state
plainly: "stub — does not register an org.a11y service or expose an Accessible tree." Remove
any wording implying a live service. No behavior change.

**Acceptance:**
```bash
cargo test -p retro-kit accessibility 2>&1 | grep 'test result:'
grep -n "stub" crates/retro-kit/src/accessibility.rs
```
**Pass condition:** existing a11y tests still `0 failed`; the grep shows the honest wording.

---

## T4 — Implement `wl_data_device` server-side send  `[Docker, needs T0, stretch]`

**Why:** `ServerDndGrabHandler::send()` in `crates/retro-compositor/src/main.rs:448-454` is an
empty body — clipboard/DnD offers accepted by the compositor deliver no data.

**Scope:** Implement `send()` to write the selection source's data for the requested
`mime_type` into the provided `fd`. Follow smithay 0.7's data_device example
(`SelectionHandler`/`DataDeviceHandler`). If the full path is too large for this pass, at
minimum wire `SelectionHandler::send_selection` for the primary/clipboard selection and leave
drag-DnD as a documented TODO — but say exactly what you did.

**Acceptance:** `docker build -t retroshell .` → `DOCKER_EXIT=0` and
`grep 'Compiling retro-compositor' /tmp/t4.log` ≥ 1 (compositor still compiles). Runtime DnD
verification is not required in this pass; note it as follow-up.

---

## T6 — Reconcile all docs with reality  `[docs only, run LAST]`

**Why:** README / ARCHITECTURE / KEYBOARD_SHORTCUTS / CONFIGURATION and any scoring text must
not repeat the three false/misleading claims the audit found.

**Files:** `README.md`, `docs/ARCHITECTURE.md`, `docs/KEYBOARD_SHORTCUTS.md`,
`docs/CONFIGURATION.md` (adjust to whatever exists).

**Changes — align every doc to the post-T2/T3/T5 code:**
1. **Compositor claim:** State accurately — RetroShell ships a *separate* smithay-based
   **nested-X11** compositor (`retro-compositor`) that the entrypoint prefers, falling back to
   labwc; `retro-shell` itself is a winit/wgpu client that renders the desktop into one
   surface. Do not say "no longer a Wayland client".
2. **Screen lock:** describe the real T2 behavior (password gate; PAM only if T2 wired it).
3. **Themes:** list the real 8 (T3).
4. **Accessibility:** describe it as role-name metadata only, no live AT-SPI service.

**Acceptance:**
```bash
grep -rniE "PAM-backed|press any key to unlock|Solarized, Dracula|no longer a Wayland client" README.md docs/
```
**Pass condition:** returns nothing (all stale claims gone). Report the (empty) output.

---

## Definition of done (whole plan)

- [ ] T0: `docker build` exits 0; `retro-compositor` binary in image. *(evidence pasted)*
- [ ] T1: runtime logs show `retro-compositor is running`, shell connected. *(logs pasted)*
- [ ] T2: wrong password stays locked; new tests pass; no "any key" string.
- [ ] T3: 8 themes, round-trip tests pass, workspace builds.
- [ ] T4: compositor still compiles with `send()` implemented (or scoped TODO documented).
- [ ] T5: a11y wording honest; a11y tests pass.
- [ ] T6: stale-claims grep returns empty.
- [ ] Final gate (one agent): `cargo test --workspace --exclude retro-compositor` → `0 failed`
      and `docker build -t retroshell .` → `DOCKER_EXIT=0`. Paste both.

## Explicitly OUT OF SCOPE (do not attempt — cost/architecture)

Full AT-SPI2 protocol; multi-monitor / `wlr_output_management`; XWayland; HDR/VRR; GPU-composited
internal widgets; making `retro-shell` a true per-window-surface client; NetworkManager/PipeWire
integration; Flatpak/sandboxing; display manager / multi-user sessions. These require a rewrite
and are not what this plan funds.
