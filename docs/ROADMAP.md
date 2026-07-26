# RetroShell Roadmap — prototype to daily driver

**Status date:** 2026-07-26
**Baseline:** commit `a0479a8` + the QA fixes on `fix/qa-2026-07-26-compositor-and-panics`
**Companion docs:** `docs/QA_REPORT_2026-07-26.md` (what is broken and why),
`docs/TOOLKIT_REMEDIATION.md` (Part A detail), `docs/HDR_VRR_DESIGN.md` (Part B detail)

This is an engineering plan, not a marketing document. Every phase ends with an
**exit criterion that can be demonstrated on a machine**, because the single
biggest failure mode in this project's history has been declaring things done
that were never executed.

---

## 0. Where we actually are

Three independent facts, all verified on 2026-07-26:

1. **The compositor had never been run.** `retro-compositor` did not compile on
   Linux from 2026-07-11 to 2026-07-26. Once it compiled, it still could not
   serve a client: the nested event loop never called `dispatch_clients`, and it
   never sent `wl_surface.frame` callbacks. Both are now fixed but only
   compile-verified; the VM harness exists to actually run them.
2. **The toolkit is render-only.** `retro-kit` widgets are laid out and
   *painted*, but they do not handle input. `Button::draw()` computes colors and
   discards them; `Button::handle_event()` swallows every left click without a
   hit test and has no activation callback. Apps look correct because each app
   hand-rolls its own hit-testing (`if self.install_button.rect().contains(point)`)
   and the SDK's downcast painter draws the widgets. Both statements are true at
   once: **the apps work, the toolkit does not.**
3. **The pure layer is genuinely good.** 545 passing tests cover output-layout
   parsing, workspace state, damage math, MIME planning, notification
   bookkeeping, idle/session policy, and package-manager argv construction. No
   command injection anywhere. This is real engineering and it is the reason the
   rest is recoverable.

The pattern to break: **a well-tested pure planner paired with a thin live
wrapper nobody ever executed.** Every phase below is designed to make the
wrapper the thing under test.

---

## Phase 0 — Make truth measurable  *(days; partly done)*

Nothing else on this list is trustworthy until this exists.

| Task | Detail | State |
|---|---|---|
| Linux CI | `.github/workflows/ci.yml` — build (`--all-targets`), test, clippy, plus a release-build gate and a non-blocking fmt check. **This alone would have caught the six-week build break.** | DONE |
| VM harness | `packaging/vm/` — VirtualBox + Arch, VMSVGA (vmwgfx = real KMS + render node), SSH port-forward for scripted QA | DONE |
| Screenshot QA | Scripted scenario runs producing artifacts under `qa/`, driven over SSH | DONE (both paths; see `docs/screenshots/vm-drm-composited.png`) |
| Ban self-scoring | Delete the score tables from `README.md`, `WARPATH_SCORECARD.md`, `DEEP_AUDIT_90_CLAIM.md`. Replace with a capability matrix whose every row cites a test name or a QA artifact. | TODO |

**Exit criterion:** a green CI badge, and `packaging/vm/qa-vm.sh` producing
screenshots of the real compositor without manual steps.

---

## Phase 1 — The compositor is real  *(1–2 months)*

Goal: `retro-compositor` hosts `retro-shell` plus two app clients, with real
buffers, correct z-order, working input, on both the DRM/KMS and nested paths.

### 1.1 Verify the three landed fixes
- `dispatch_clients` in the nested loop (`main.rs`)
- `wl_surface.frame` callbacks after present
- front-to-back render element order for `draw_render_elements`
**Exit:** a screenshot of a real client window rendered by `retro-compositor`.

### 1.2 Replace the dumb-buffer scanout with GL composition (DRM path) — **DONE 2026-07-26**
Landed: `DrmCompositor` over the scanout surface, elements collected from
layer-shell chrome + workspace-filtered windows, `render_frame`/`queue_frame`
paced by a calloop vblank handler calling `frame_submitted()`, and
`on_commit_buffer_handler` wired into the DRM `commit` (its absence was why
buffers never became renderable). Verified by
`docs/screenshots/vm-drm-composited.png`. Remaining in this area:
- damage-driven redraw instead of rendering every tick (use the
  `RenderFrameResult` damage and `need_full_redraw` rather than unconditional
  `render_frame`)
- reschedule on `FrameError::EmptyFrame` with a one-shot timer (~one retrace)
  instead of spinning
- direct scan-out of client buffers: pass a real `import_node` to
  `GbmFramebufferExporter::new` so suitable client dmabufs go straight to a
  plane
- multi-output: one `DrmCompositor` per CRTC, keyed by `crtc::Handle` in the
  vblank handler
**Exit:** damage-tracked composition across two outputs with no full redraws.

### 1.3 Input correctness — *in progress*
Landed: libinput keyboard/pointer reach the seat; `cursor_image` stores the
`CursorImageStatus` and the client cursor surface is drawn topmost with
`Kind::Cursor` so it can reach the hardware cursor plane. Remaining:
- **named cursors**: load an XCursor theme so `CursorImageStatus::Named`
  renders. Until then a client that never sets a surface has no pointer.
- pointer axis/scroll, touch, tablet
- keyboard repeat, XKB layout from config rather than `XkbConfig::default()`
- **Diagnose why keyboard never reaches `retro-shell` under labwc** (QA finding B)
**Exit:** typing works in a terminal client under `retro-compositor`; the mouse
cursor is visible and moves.

### 1.4 Window management protocol surface
- `xdg_toplevel` interactive move/resize (`move_request`, `resize_request`) —
  the shell paints its own chrome, so the compositor must implement grabs
- `xdg_popup` positioning with constraint adjustment
- `wl_output` scale/transform changes at runtime
**Exit:** dragging a client window's title bar moves it; a menu popup appears in
the right place and dismisses correctly.

---

## Phase 2 — The toolkit is real  *(2–3 months)*

This is the largest single body of work and the prerequisite for every app
improvement. Full detail in `docs/TOOLKIT_REMEDIATION.md`; summary here.

### 2.1 Hit-test dispatch
A generic tree walk that finds the deepest visible, enabled widget whose rect
contains the point and delivers the event there, bubbling on `Ignored`.
```rust
// retro-kit/src/dispatch.rs
pub fn dispatch_pointer(root: &mut dyn Widget, ev: &Event, at: Point) -> EventResult;
pub fn widget_at(root: &dyn Widget, at: Point) -> Option<WidgetId>;
```
Every widget's `handle_event` must stop assuming it was targeted correctly, and
must stop returning `Handled` for events outside its rect (the current `Button`
bug).

### 2.2 Focus system
`WidgetState.focused` already exists and nothing sets it. Add:
```rust
pub struct FocusManager { focused: Option<WidgetId>, order: Vec<WidgetId> }
impl FocusManager {
    pub fn focus(&mut self, root: &mut dyn Widget, id: WidgetId);
    pub fn focus_next(&mut self, root: &mut dyn Widget);   // Tab
    pub fn focus_prev(&mut self, root: &mut dyn Widget);   // Shift+Tab
    pub fn deliver_key(&mut self, root: &mut dyn Widget, ev: &Event) -> EventResult;
}
pub trait Widget { fn focusable(&self) -> bool { false } /* ... */ }
```
This fixes the "two `TextField`s in one window both eat every keystroke" bug and
is what `keyboard_nav.rs` in the shell has been approximating.

### 2.3 Activation callbacks
```rust
pub struct Button { on_click: Option<Box<dyn FnMut() + Send>>, pressed: bool, /* ... */ }
impl Button { pub fn on_click(self, f: impl FnMut() + Send + 'static) -> Self; }
```
with correct press/release semantics (activate only if release lands inside the
same widget as the press).

### 2.4 Per-widget remediation
`Button`, `PopupButton`, `TabView`, `ScrollView`, `Dialog`, `ListView`,
`TreeView`, `Slider`, `Toolbar`, `MenuBar`, `IconView`, `SplitView`,
`StatusBar`, `MonospaceView`, `DockView`, `WorkspaceGridView`. Each needs: a
real `draw()` **or** an explicit documented decision that the SDK painter owns
it, a rect-checked `handle_event`, and an activation path.

### 2.5 Kill the downcast painter
`retro-sdk`'s `draw_widget` is a 1200-line `if let Some(x) = w.as_any().downcast_ref::<T>()`
chain — every new widget requires editing the SDK, and `Widget::draw()` is dead
weight. Move painting into the widgets behind a `Canvas` handle passed to
`draw(&self, canvas: &mut Canvas, theme: &ThemeContext)`.

**Migration order that keeps apps working:** add dispatch + focus alongside the
existing paths (2.1–2.3) → port one app (Settings, the most widget-dense) → port
the rest → delete app-level hit-testing → then 2.5.

**Exit:** Settings is fully keyboard-navigable with Tab, every control is
clickable through generic dispatch, and no app contains a `rect().contains(point)`
call.

---

## Phase 3 — Session integrity  *(1–2 months)*

### 3.1 A lock screen that actually locks
Today the "lock" only covers the shell's own surface — a client launched while
locked draws over it and receives input (QA finding A, screenshot evidence).
Required:
- implement **`ext-session-lock-v1`** in `retro-compositor`
- on lock: the compositor gives the lock surface exclusive keyboard/pointer
  focus, refuses focus to every other surface, and blanks other outputs
- the shell becomes a lock client rather than painting a lock window
- hash the password (argon2) instead of comparing a plaintext `settings.conf`
  value; ideally authenticate via PAM
**Exit:** with the session locked, launching a client and typing produces
nothing until the correct password unlocks.

### 3.2 Portals that a real client can use
`org.retroshell.Portal` is a private bus name with non-spec signatures and no
`.portal` backend file, so no GTK/Qt app can reach it. Screenshot returns
success for a file it never wrote; `option_string_loose` parses zvariant `Debug`
output. Rebuild against the real `org.freedesktop.impl.portal.*` interfaces with
correct signatures, register a backend file, and implement Screenshot on top of
the compositor rather than shelling out to X11 `import`.

### 3.3 Idle, power, and inhibit
Inhibit cookies are never released when a client dies (permanently disabling
auto-lock). Track cookies per-connection and release on `NameOwnerChanged`.

---

## Phase 4 — Display pipeline: HDR, VRR, multi-monitor  *(2–3 months)*

Detail in `docs/HDR_VRR_DESIGN.md`.

### 4.1 Real capability detection *(landing now)*
Replace `HdrCapabilities::detect()`'s hardcoded `false` with actual DRM
connector property enumeration: `HDR_OUTPUT_METADATA`, `Colorspace`, `max bpc`,
`vrr_capable`. Parse EDID CTA-861 HDR Static Metadata for the display's real
luminance range.

### 4.2 Real application
- HDR: build the `hdr_output_metadata` blob (ST2084/PQ EOTF, mastering display
  primaries, MaxCLL/MaxFALL), set it on the connector, set `Colorspace` to
  `BT2020_RGB`, raise `max bpc` to 10.
- VRR: set the CRTC `VRR_ENABLED` property when the connector reports
  `vrr_capable` and the user enabled it; switch the frame scheduler from
  fixed-deadline to present-when-ready within the EDID's refresh range.

### 4.3 Colour-correct composition
A 10-bit/FP16 render target, sRGB→PQ transfer on composition, and a real
tone-mapping curve to replace the current placeholder `ToneMapper`. This is
where "HDR" becomes visually true rather than merely signalled.

### 4.4 Wayland colour management
Implement `wp_color_management_v1` so clients can describe their content's
colour volume, rather than the compositor guessing.

### 4.5 Multi-monitor for real
`display_arrange` currently sets an env var **inside the wrong process** and
reports a fabricated `1920x1080 eDP-1`. Replace with live KMS modeset:
enumerate connectors, apply user arrangement via atomic commit, handle hotplug
through the existing udev source.

**Exit:** on your Arch box, `retro-compositor` reports your monitor's true HDR
capability, the toggle changes the connector property (verifiable with
`sudo modetest -c | grep -A5 HDR_OUTPUT_METADATA`), and VRR shows measurable
variable frame pacing.
*A VM cannot verify this* — vmwgfx exposes neither property. The VM verifies the
code paths execute and degrade honestly.

---

## Phase 5 — App platform  *(ongoing, gated on Phase 2)*

- **Terminal**: complete the VT parser — cursor-movement CSIs (A/B/C/D/G/d),
  ED 0/1, HT, and make DECSTBM scroll margins actually affect LF. Today `vim`
  and `less` will corrupt the screen. Reap child processes on tab close.
- **Finder**: cross-filesystem trash (currently `fs::rename` only, silently
  fails across mounts), file-operation progress, undo.
- **TextEdit**: real text layout (selection, wrapping, multi-cursor) once the
  toolkit has a focus system.
- **Settings**: atomic `settings.conf` writes (current read-modify-write loses
  keys under concurrency), and live-apply for display settings.
- **AppStore**: async package operations with real progress from the package
  manager rather than a fake indeterminate bar.

---

## Phase 6 — Ecosystem compatibility  *(3–6 months)*

This is what separates "a desktop" from "a desktop you can use."

- **XWayland**: currently best-effort and unmanaged. Needs real X11 window
  management: map/unmap, stacking, override-redirect, selections, DPI.
- **Third-party toolkits**: GTK4 and Qt6 apps must run correctly — that means
  correct `xdg_shell` semantics, working portals (file chooser, screenshot,
  settings), `xdg-decoration`, clipboard, and DnD.
- **Global menus for foreign apps**: the current global menu only works for
  first-party apps that publish manifests. Real support means the AppMenu D-Bus
  protocol (`com.canonical.dbusmenu`), as KDE does.
- **Display manager**: a greeter has never been run. Either integrate with
  greetd/SDDM or ship one.
- **Input methods**: IME exists compositor-side but there is *no client-side IME
  path in the SDK*, so no RetroShell app can accept CJK input.

---

## Phase 7 — Daily-driver hardening  *(continuous)*

- Crash recovery: the shell dying should not kill the session.
- Memory/leak discipline: the DRM framebuffer leak (fixed) is the kind of thing
  that must be caught by a soak test, not an audit.
- Performance: today the shell busy-polls, re-reads `settings.conf` from disk
  every frame, and makes blocking D-Bus + subprocess calls on the render thread
  every 5 seconds. Fix with inotify + a background status thread.
- Accessibility: the AT-SPI tree exists but is not Orca-complete.
- Localisation, HiDPI fractional scaling, session restore.

---

## What "rivalling KDE/GNOME" actually means

Being straight about scope, because it affects how you plan:

| | KDE Plasma | GNOME | RetroShell today |
|---|---|---|---|
| Age | 26 years | 27 years | ~2 months of real work |
| Contributors | ~500/yr active | ~300/yr active | 1 |
| Toolkit | Qt (30 yrs, ~1M LOC) | GTK (27 yrs) | retro-kit, ~7k LOC, no input layer |
| Compositor | KWin (~250k LOC) | Mutter (~200k LOC) | retro-compositor, ~7k LOC |
| Apps | ~200 | ~80 | 5 |

**Feature parity is not a realistic target** and chasing it would be the fastest
way to make this project miserable. What *is* realistic, and genuinely valuable:

> **A visually distinctive, coherent, fast desktop that one developer can
> daily-drive for focused work** — terminal, editor, files, browser, settings —
> on hardware they control, with modern display support (HDR/VRR) that most
> lightweight DEs *don't* have.

That is achievable and it is a real contribution. Sway is ~50k LOC and people
daily-drive it. River, Niri, and Hyprland all found audiences without matching
KDE feature-for-feature. **The differentiator here is the Classic Mac OS design
language done properly** — nobody else is doing that on Wayland with a real
compositor.

**Realistic milestones:**

| Horizon | Milestone |
|---|---|
| ~3 months | Compositor hosts real clients reliably; toolkit is interactive; you can run a terminal and editor inside RetroShell for an hour without hitting a blocker |
| ~6 months | Session integrity (real lock, portals, greeter); GTK apps run; you can do a day's work in it |
| ~12 months | HDR/VRR real on your hardware; multi-monitor; XWayland solid; other people can install it |
| ~24 months | Stable enough to recommend; app ecosystem beyond first-party; contributors |

The 24-month version is not "KDE with a different theme." It is "a genuinely
good small desktop with a strong point of view." Aim there.

---

## How to work through this

1. **Never claim a capability without an artifact.** A test name, a screenshot,
   or a QA log line. The scorecard habit is what let three fatal compositor bugs
   survive four rounds of "audit."
2. **CI first.** It is one afternoon and it prevents the single worst failure
   this project has had.
3. **One phase at a time, in order.** Phase 2 (toolkit) is tempting to skip
   because the apps look fine. Everything in Phase 5 is blocked behind it.
4. **Prefer deleting a claim to defending it.** The README is more credible now
   that it says "unknown" than it was saying "85/100."
