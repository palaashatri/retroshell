# RetroShell — Path to KDE/GNOME-Class Polish & Workability

> **Purpose**: Roadmap for turning RetroShell from a working retro shell + first-party
> apps into a **daily-driver-class Linux desktop environment** (same *category* as
> Plasma / GNOME Shell — not a pixel clone). Written for implementers and agents:
> ordered phases, concrete modules, acceptance criteria, and honest dependencies.
>
> **Positioning**: Ambition is real. Full parity is multi-year. This document is the
> sequence of work that *actually gets there*, not marketing.
>
> **Latest competitive audit:** §13 + `docs/WARPATH_SCORECARD.md` +
> `docs/DEEP_AUDIT_90_CLAIM.md` (2026-07-11).
> **Verdict:** prior **~90 withdrawn** (score theater). After warpath live integration,
> honest overall **~85 / 100** (mean **84.5**, sum **845**). Still not 90 — greeter NOT RUN, PipeWire
> stubs, §12 **0 / 7**. Prefer under-claim.
>
> **Related**: [`README.md`](../README.md) (ambition vs reality),
> [`ARCHITECTURE.md`](ARCHITECTURE.md), [`audit_2026-07-09.md`](audit_2026-07-09.md),
> [`FULL_AUDIT_2026-07-11.md`](FULL_AUDIT_2026-07-11.md),
> [`WARPATH_SCORECARD.md`](WARPATH_SCORECARD.md).

---

## 1. What “KDE/GNOME level” means for RetroShell

Not “looks like Plasma.” **Workability** means a normal user can:

| Capability | Plasma/GNOME equivalent | RetroShell target |
|---|---|---|
| Log in and get a session | SDDM/GDM + session unit | Display manager + `retroshell-session` |
| Run arbitrary apps as windows | KWin/Mutter | Compositor owns all `xdg_toplevel`s |
| Shell chrome without painting fakes | Panel / Shell | Dock, menus, notifications as real surfaces or protocol-backed chrome |
| Files, terminal, settings | Dolphin/Nautilus, Konsole, System Settings | First-party suite + open third-party |
| Clipboard / DnD between apps | Wayland data device | Full `wl_data_device` + portals |
| Multi-monitor, scale, HDR/VRR | KScreen / Mutter | Output management + scale tree |
| Sound, network, power, notifications | PipeWire, NM, UPower, notifications | Live control, not status-only prefs |
| Accessibility | Orca + AT-SPI | Full Accessible tree + keyboard paths |
| Screenshots, screen share, portals | xdg-desktop-portal | Screenshot, file chooser, screencast |
| Secure lock / multi-user | PAM + session | Real auth + session lifecycle |

**RetroShell aesthetic** (Classic Mac / NeXT / BeOS) can remain. **Architecture** must
converge on FreeDesktop session norms or it will never feel like a real DE.

---

## 2. Where we are today (baseline, 2026-07-11)

| Layer | Status | Gap vs DE-class |
|---|---|---|
| Toolkit (`retro-kit` / `retro-render` / `retro-sdk`) | Strong: widgets, wgpu, themes, menus | No scale-factor tree; limited a11y roles |
| First-party apps | Real I/O (Finder, Terminal, TextEdit, Settings, App Store) | Not all FreeDesktop portals; limited third-party |
| Shell chrome | Dock/menu painted for visuals; **layer-shell client** maps bar/dock namespaces when `WAYLAND_DISPLAY` live | Dual path: kit paint still drives UI; protocol chrome is real bind when compositor supports zwlr_layer_shell |
| Multi-client apps | Process spawn + **ext-foreign-toplevel-list client** sync into Force Quit | FTL client best-effort; still also tracks PIDs in session registry |
| Compositor (`retro-compositor`) | Nested: SHM + **layer-shell in render_frame**; FTL; decorations; DRM: modeset + dumb-buffer **commit/page_flip** present path | Full multi-plane composition / damage still progressive; Docker often labwc |
| System integration | NM status + connect plan, volume, power, FDO notifications, **portal D-Bus** Screenshot/Settings/OpenURI | No full polkit UI, IME, screencast, greeter proof |
| Packaging | `packaging/*.desktop`, `start-retroshell`, Docker + noVNC | Skeleton; greeter→session **not proven** on hardware |

**Competitive score (honest, vs Plasma/GNOME daily driver):** see **§13** +
`docs/WARPATH_SCORECARD.md` + `docs/GOAL_DEEP_AUDIT_FINAL.md` — overall **~85**
(mean **84.5**, sum **845**). Prior ~90 claim **withdrawn**.

**Architectural bottleneck (must solve early):**  
`retro-shell` is still largely a **single fullscreen winit client** that *draws* an
internal desktop. KDE/GNOME-level workability requires the **compositor** to manage
app surfaces, and the shell to become a **session client** (panels + protocols), not
a fake multi-window renderer.

---

## 3. Target architecture (end state)

```
┌─────────────────────────────────────────────────────────────┐
│  Display Manager (greeter) → starts user session            │
└────────────────────────────┬────────────────────────────────┘
                             ▼
┌─────────────────────────────────────────────────────────────┐
│  retro-compositor  (Smithay, DRM/KMS primary, X11 nested    │
│    optional for nested/dev)                                 │
│  - xdg_shell, layer_shell, seat, data_device, output_mgmt  │
│  - XWayland, presentation / color management when available │
└───────────────┬─────────────────────────────┬───────────────┘
                │                             │
     ┌──────────▼──────────┐       ┌──────────▼──────────────┐
     │  retro-shell        │       │  Apps (first-party +    │
     │  (layer-shell bar,  │       │   third-party Wayland   │
     │   dock, desktop,    │       │   / XWayland clients)   │
     │   notifications)    │       └─────────────────────────┘
     └──────────┬──────────┘
                │ D-Bus / portals
     ┌──────────▼────────────────────────────────────────────┐
     │  session services: portal, AT-SPI, NM, PipeWire,      │
     │  UPower, notification daemon, polkit agent            │
     └───────────────────────────────────────────────────────┘
```

**Non-negotiable rule:** new features must not deepen the “everything is a rect
inside shell” model unless explicitly temporary and scheduled for deletion.

---

## 4. Phased roadmap

Phases are sequential where marked **BLOCKING**. Parallelize only within a phase
when file ownership does not collide (`retro-compositor/main.rs` stays single-threaded
ownership).

### Phase A — Session foundation (BLOCKING for everything else)

**Goal:** Own a real Linux session end-to-end on hardware (Pi / desktop GPU).

| Work item | What to build | Primary files / artifacts | Done when |
|---|---|---|---|
| A1 DRM/KMS backend | Smithay `backend_drm` / udev seat path alongside nested X11; auto-select backend | `crates/retro-compositor/` Cargo features, `main.rs` | Boots on bare metal without Xvfb; logs show DRM outputs |
| A2 Software / llvmpipe path | Documented fallback for CI/Docker without DRI3 (honest labels) | entrypoint, README | Docker either runs compositor *or* explicitly labwc; never silent lie |
| A3 Session unit | `retroshell.desktop` + systemd user target or `start-retroshell` | `packaging/`, `scripts/` | `startx`/`cage`-style or DM can start session |
| A4 Display manager hook | Greeter session entry (LightDM/SDDM/GDM custom session) | `packaging/*.desktop` | Login → shell + compositor without manual docker |
| A5 Seat / logind | `libseat` / logind integration for VT and device ACLs | compositor | Multi-user switch / clean logout without hung DRM |

**Exit criterion:** On a Pi or x86 GPU box, cold boot → greeter → RetroShell session
usable without Docker. Nested Docker remains a **dev** path, not the definition of
“works.”

---

### Phase B — True multi-client windowing (BLOCKING for DE polish)

**Goal:** Every app window is a compositor surface; shell chrome is not a fake WM.

| Work item | What to build | Primary files | Done when |
|---|---|---|---|
| B1 Finish client lifecycle | Map/unmap/configure/close for `xdg_toplevel`; activate/focus; decorations policy | `retro-compositor` | Two clients (finder + settings) stack/focus under **retro-compositor**, not only labwc |
| B2 Shell stops painting app windows | Remove in-shell “app window” chrome for external processes; shell only tracks PIDs + activation | `retro-shell` `ShellDesktop`, `window_manager` | Force Quit / Alt-Tab operate on compositor clients |
| B3 Layer shell for chrome | `wlr-layer-shell` (or equivalent) for menu bar, dock, notifications | compositor + shell | Bar/dock are separate surfaces; not overdrawn inside shell canvas |
| B4 Desktop / wallpaper surface | Dedicated layer or background surface | shell + compositor | Desktop icons on a real surface or protocol-backed desktop |
| B5 Foreign toplevel list | `wlr-foreign-toplevel` / ext-foreign-toplevel for task list, Force Quit, overview | compositor + shell | Shell lists *all* toplevels without guessing paint-rects |
| B6 XWayland polish | Map, focus, clipboard bridge complete for X11 apps | compositor | Firefox/LibreOffice-class X11 apps usable |
| B7 Input grabs & popups | Correct xdg_popup / grab chains for menus | compositor + kit | Menus don’t steal wrong focus or clip incorrectly |

**Exit criterion:** User launches Finder + Terminal + a third-party Wayland app;
all three are independent windows with correct focus, z-order, resize, and close
under `retro-compositor` on hardware.

**Code already started (build on this):**

- `ClientWindowStack` — map/focus/z-order policy (`crates/retro-compositor/src/lib.rs`)
- `SessionClientRegistry` / `spawn_app_client` — process tracking (`session_clients.rs`)
- Selection send, multi-output env, XWayland spawn path in compositor

---

### Phase C — Shell chrome at DE quality

**Goal:** Dock, menus, notifications, workspaces feel like a finished product.

| Work item | Notes | Done when |
|---|---|---|
| C1 Global menu for external apps | Compositor/session ownership of app menus (or AppMenu protocol / GTK/Qt integration) | Non-RetroShell apps show menus in bar when possible |
| C2 Workspace model | Tie workspaces to compositor (not only shell-filtered paint list) | Switching workspaces hides/shows real surfaces |
| C3 Overview / app switcher | Cmd+Tab across foreign toplevels; optional expose | Matches muscle memory of DE users |
| C4 Notification daemon | FreeDesktop Notifications spec (org.freedesktop.Notifications) | `notify-send` works; banners are real surfaces |
| C5 Lock screen | PAM or systemd-logind lock; block input at compositor | Secure lock, not app-level only |
| C6 HiDPI | Scale factor through kit layout + compositor buffer scale | 2× displays crisp; settings control |
| C7 Theming pipeline | Export theme tokens to apps; runtime reload | One settings change updates shell + first-party apps |
| C8 Settings depth | Real NM connect, audio devices, displays, users, default apps | System Settings-class coverage for daily use |

**Exit criterion:** New user can configure Wi‑Fi, volume, display scale, theme,
and notifications without reading source.

---

### Phase D — FreeDesktop integration (workability with the Linux ecosystem)

**Goal:** “It runs my apps and plays well with others.”

| Work item | Protocol / service | Done when |
|---|---|---|
| D1 xdg-desktop-portal | FileChooser, Screenshot, Screencast, Settings, OpenURI | Flatpak/browser file pickers work |
| D2 PipeWire integration | Screencast + pro audio path | Screen share in browsers; multi-device mixer basics |
| D3 NetworkManager full | Connections, Wi‑Fi secrets via agent | Connect to WPA2 network from UI |
| D4 Power / session | Inhibit, suspend, lid, battery actions | Laptop lid / suspend policy works |
| D5 Polkit agent | Auth dialogs for privileged actions | `pkexec` / software install prompts |
| D6 MIME + .desktop | Full XDG app discovery beyond builtins | “Open with” for system-installed apps |
| D7 Clipboard managers | Persistent clipboard + primary selection | Cross-app clipboard after restart |
| D8 Input methods | ibus/fcitx Wayland IM protocol | CJK/IME usable |

**Exit criterion:** Install Firefox (or equivalent) from distro packages; browse,
download file via portal, print optional, clipboard to Terminal works.

---

### Phase E — Accessibility & internationalization

| Work item | Done when |
|---|---|
| E1 Full AT-SPI tree for kit widgets | Orca announces buttons, focus, text fields |
| E2 Keyboard-only UI | Full desktop operable without pointer |
| E3 High contrast / reduce motion | Settings map to real behavior |
| E4 i18n (gettext/fluent) | Non-English UI strings for shell + first-party apps |
| E5 RTL layout | At least basic RTL for shell chrome |

**Exit criterion:** Orca user can launch apps, navigate Settings, and lock/unlock
with speech feedback.

---

### Phase F — Polish, performance, packaging

| Work item | Done when |
|---|---|
| F1 Frame budget | 60 Hz shell chrome on Pi 4/5 class hardware with soft animations optional |
| F2 Startup time | Session interactive &lt; 5 s on reference hardware after login |
| F3 Crash recovery | Compositor or shell crash restarts without full reboot |
| F4 Distro packages | `.deb` / Fedora COPR / Arch PKGBUILD + session file |
| F5 Flatpak runtime docs | Document portals + permissions for third-party |
| F6 QA matrix | Automated: unit + compositor integration + golden screenshots + Pi CI |
| F7 Security | Sandbox story, no secrets in image ENV, secure defaults for lock |

**Exit criterion:** Installable from a package, bootable session, CI green on
hardware job (not only Docker labwc).

---

## 5. Recommended execution order (critical path)

```
A1 DRM compositor ──► B1–B2 multi-client under our compositor
        │                        │
        ▼                        ▼
A3–A5 session/DM          B3–B5 layer-shell + foreign-toplevel
        │                        │
        └──────────┬─────────────┘
                   ▼
            C shell chrome quality
                   │
         ┌─────────┴─────────┐
         ▼                   ▼
    D portals/OS         E a11y/i18n
         │                   │
         └─────────┬─────────┘
                   ▼
              F polish/packaging
```

**Do not** invest heavily in new in-shell paint widgets for “apps” before B1–B2.
That is throwaway work against the DE goal.

**Do** keep labwc as a **compatibility/dev** compositor while A1/B1 land, but
measure success on **hardware + retro-compositor**.

---

## 6. Module-level work breakdown (implementer map)

### `crates/retro-compositor`
- Backend split: `backend_x11` (nested) vs `backend_drm` (session)
- `ClientWindowStack` → live wiring to real `ToplevelSurface` map/unmap/focus
- `wlr-layer-shell`, foreign-toplevel, output management
- Complete data_device + primary selection + DnD sources
- XWayland complete path; decoration protocol (server/client side policy)
- Presentation / color management when stacking HDR for real

### `crates/retro-shell`
- Split: **session shell** (layer surfaces) vs **legacy single-surface** (delete over time)
- `SessionClientRegistry` ↔ foreign-toplevel sync
- Force Quit / Alt-Tab / dock attention from real client list (already partially there)
- Notification daemon (D-Bus) instead of in-process only
- PAM/logind lock integration
- Settings panes backed by Phase D services

### `crates/retro-kit` / `retro-render` / `retro-sdk`
- Logical pixel + scale factor through layout and text
- Widget a11y nodes on every interactive control
- App template: “first-party app” = proper Wayland client with portal usage

### `apps/*`
- Prefer standard dialogs via portals over custom path pickers long-term
- Terminal: keep PTY strength; add tabs/profiles polish
- Finder: volume mount UX, trash, search
- Settings: display topology, network connect, default applications

### Packaging / session
- `packaging/retroshell.desktop`, systemd user units, polkit rules
- `scripts/verify_pi.sh` expanded to session smoke (not only unit tests)

---

## 7. Acceptance gates per phase (no theater)

For each phase, **all** of:

1. **Host tests** for pure policy (`cargo test --workspace --exclude retro-compositor` or package-local).
2. **Linux build** of compositor (`docker build` and/or Pi).
3. **Behavioral proof**: screenshot or process list + logs on the *target* path
   (hardware for session; labwc only as secondary).
4. **Docs** updated so README never claims a phase is done without the proof.

Example Phase B gate:

```bash
# On Linux with GPU session (not only mac Docker):
# 1) retro-compositor is the session compositor
# 2) ps shows retro-shell + finder + terminal
# 3) logs show map/focus of distinct wl surfaces
# 4) cargo test -p retro-compositor && cargo test -p retro-shell
```

---

## 8. Effort reality (planning only)

| Phase | Rough effort (small focused team) | Parallelism |
|---|---|---|
| A Session foundation | 4–8 weeks | Low (compositor serial) |
| B Multi-client windowing | 2–4 months | Medium after A1 |
| C Shell chrome quality | 2–3 months | High (UI + protocols) |
| D FreeDesktop integration | 3–6 months | High (services) |
| E A11y / i18n | 2–4 months | Medium |
| F Polish / packaging | Ongoing | High |

A single agent sprint can deliver **slices** (e.g. B1 helpers + tests, D2 volume
already landed). **Whole DE-class workability** is the sum of phases A–F.

---

## 9. Completed foundation (do not re-do)

Prior sprints already delivered usable building blocks. Treat as done unless
regressing:

- Password lock + pure `verify_lock_password`; conf merge preserves `lock_password`
- Eight themes in shell + Settings
- Compositor selection send, multi-output env, display policy, XWayland **path**
- NM status, volume get/set, UPower/`sys`, screenshot/record menus
- AT-SPI Accessible export (minimal tree)
- Multi-client process spawn + Force Quit window/client kill path
- Docker noVNC image with labwc fallback when DRI3 missing
- Pi verification script skeleton (`scripts/verify_pi.sh`)

Evidence trails: `docs/audit_2026-07-11.md`, `docs/FULL_AUDIT_2026-07-11.md`,
git history on `fix/compositor-build-and-audit`.

---

## 10. Rules of engagement for future agents

1. **No fabrication** of test/build results.
2. **Evidence or it didn’t happen** (commands + outputs).
3. Prefer **protocol/session** solutions over more in-shell paint fakes.
4. Compositor Linux-only: verify with Docker **and** Pi/hardware for session claims.
5. Every milestone: tests for pure helpers + one integration/smoke path.
6. Update this document’s phase checkboxes when a phase exit criterion is met.

---

## 11. Phase checklist (track progress)

**Legend:** **In tree** = code/packaging artifact exists · **Verified** = proven on the
target path with command/output evidence (hardware for session claims; dual-client
under `retro-compositor` where required). A bare historic `[x]` without Verified does
**not** mean Phase exit criteria are met.

### Phase A — Session foundation
| ID | Work | In tree | Verified | Notes |
|---|---|---|---|---|
| A1 | DRM/KMS backend | **yes** | **no** | `session_drm::run_drm_session` libseat+udev+libinput+GBM/EGL+protocol loop; **no pageflip/scanout**; untested on real seat |
| A2 | Nested/labwc honest fallback | **yes** | **yes** (Docker labwc path prior audits) | `start-retroshell`, entrypoint, `session_mode_summary` |
| A3 | Session launcher / user unit | **yes** | partial | `scripts/start-retroshell`, `packaging/retroshell.service` skeleton |
| A4 | DM session `.desktop` | **yes** | **no** | Files installable; greeter→session not QA'd |
| A5 | seatd/logind (libseat) | **yes** | **no** | Open/pause/activate in DRM path; multi-user/logout unproven |

**Phase A exit (cold boot → greeter → session without Docker):** **not met.**

### Phase B — Multi-client windowing
| ID | Work | In tree | Verified | Notes |
|---|---|---|---|---|
| B1 | Multi-client map/focus + process spawn | **yes** | partial | `ClientWindowStack` + dual-client smoke under **labwc**; not proven under `retro-compositor` on hardware |
| B2 | Shell not fake-WM for external apps | partial | **no** | External apps are processes; shell still paints internal windows |
| B3 | Layer-shell bar/dock | **server yes / shell no** | **no** | Globals+handlers on nested+DRM; shell chrome still in-canvas |
| B4 | Desktop surface | **no** | **no** | |
| B5 | Foreign toplevel list | **yes** | partial | Handles on map/title/app_id/close; Force Quit still PID/title registry |
| B6 | XWayland path | **yes** (nested) | partial | Nested spawn+WM; DRM path has no XWayland; rootless polish open |
| B7 | Popups/grabs | **no** | **no** | |

**Phase B exit (Finder+Terminal+third-party under own compositor on hardware):** **not met.**

### Phase C — Shell chrome quality
| ID | Work | In tree | Verified | Notes |
|---|---|---|---|---|
| C1 | External global menu | partial | **no** | First-party menus; not arbitrary apps |
| C2 | Workspaces | shell yes | partial | Shell filter; not compositor workspaces |
| C3 | App switcher / overview | **no** | **no** | |
| C4 | FDO notifications | **yes** | partial | zbus when bus present |
| C5 | Lock | app yes | partial | Password gate; not compositor/PAM lock |
| C6 | HiDPI scale tree | **no** | **no** | |
| C7 | Themes + Settings conf | **yes** | **yes** (unit) | 8 themes, conf merge |
| C8 | Settings depth MVP | **yes** | partial | Volume apply, NM status, display prefs |

### Phase D — FreeDesktop
| ID | Work | In tree | Verified | Notes |
|---|---|---|---|---|
| D1 | xdg-desktop-portal | partial | **no** | `portal.rs` screenshot API facade only — **not** portal bus |
| D2 | Volume (Pulse/PipeWire CLI) | **yes** | partial | pactl/wpctl |
| D3 | NetworkManager | status yes | partial | No connect UI |
| D4 | Power status | **yes** | partial | UPower/`sys` |
| D5 | Polkit agent | **no** | **no** | |
| D6 | Full .desktop/MIME | **no** | **no** | |
| D7 | Clipboard / selection | **yes** | partial | kit + compositor send |
| D8 | IME | **no** | **no** | |

### Phase E — A11y / i18n
| ID | Work | In tree | Verified | Notes |
|---|---|---|---|---|
| E1 | AT-SPI export | minimal | partial | Not Orca-complete |
| E2 | Keyboard-only | **no** | **no** | |
| E3 | Contrast/motion prefs | partial | **no** | |
| E4 | i18n | **no** | **no** | |
| E5 | RTL | **no** | **no** | |

### Phase F — Polish / packaging
| ID | Work | In tree | Verified | Notes |
|---|---|---|---|---|
| F1 | Perf budget on hardware | **no** | **no** | |
| F2 | Startup budget | **no** | **no** | |
| F3 | Crash recovery | **no** | **no** | |
| F4 | Session packaging skeleton | **yes** | partial | |
| F5 | Flatpak guidance | **no** | **no** | |
| F6 | Host tests + Docker smoke | **yes** | partial | Host pure tests green; **Docker image `retroshell:drm-session-v2` built 2026-07-11** (workspace release compile with DRM features + session_drm — build exit 0, image `d0b02f605593`); runtime DE smoke under labwc not re-run this pass |
| F7 | Honest DRI3/labwc docs | **yes** | partial | README residual gaps need refresh after DRM code land |

---

## 12. Definition of “we made it”

RetroShell is **KDE/GNOME-class in workability** when **all** are true:

1. Install from a distro package; log in via a greeter into a RetroShell session.
2. `retro-compositor` is the session compositor on real GPU hardware.
3. Arbitrary Wayland (and common X11) apps run as first-class windows.
4. Shell chrome (bar, dock, notifications, lock) uses real session protocols.
5. Network, sound, power, screenshots, and file open/save work via standard stack
   (NM, PipeWire, portals) for daily tasks.
6. Orca can drive core UI; keyboard-only is possible.
7. Docs match reality; CI proves the above on hardware, not only nested Docker.

**§12 status as of 2026-07-11: 0 / 7 fully met.** Until then: ship incrementally,
stay honest, and follow the critical path above.

---

## 13. Competitive audit vs KDE Plasma / GNOME (skeptic-fixed after 90 claim)

> **Methodology (unchanged):** domain scores and **overall** = workability vs
> **Plasma/GNOME as a daily-driver laptop DE** (100 = replace Plasma for a week).
> **Pure modules + unit tests without live wiring do not score as 90.**
> Equal weight: mean of 10 domains; round mean to nearest integer for “~NN”.
>
> Full write-up: **`docs/DEEP_AUDIT_90_CLAIM.md`**, arithmetic:
> **`docs/WARPATH_SCORECARD.md`**.

### 13.1 Verdict

Hard-DE **code** landed, then a deep audit rejected the ~90 claim as score theater
(integration gaps). A **warpath** same day closed several live paths (workspace paint
in compositor `main`, SHM prefer, `RETROSHELL_OUTPUTS_LAYOUT`, DoAction drain, i18n
menus, portal Inhibit→idle, install-session + daily checklist, menu/dock/desktop
a11y windows). Residuals remain: **live greeter NOT RUN**, PipeWire ScreenCast
**stubs**, Orca incomplete, §12 **0 / 7**.

**Prior overall ~90 (mean 89.6) is WITHDRAWN as score theater.**  
Timeline under original methodology: claim audit **~76** → first wire **~77** →
warpath **~85 (mean 84.5, sum 845)**. Still not 90. Prefer under-claim.

### 13.2 Scorecard (warpath, honest — **canonical vector only**)

| Domain | Inflated (~90 card) | Post-claim (~77) | **Warpath honest** | Why |
|---|---:|---:|---:|---|
| First-party productivity apps | 90 | 86 | **88** | Suite + MIME open + Force Quit |
| Toolkit / look & feel | 90 | 80 | **84** | MenuBar open API + DoAction queue |
| Session login / packaging | 88 | 74 | **78** | install-session + checklist; greeter **NOT RUN** |
| Own compositor as session WM | 90 | 80 | **86** | Workspace paint/focus; SHM prefer; damage stats |
| Multi-client window management | 90 | 76 | **83** | Live ws hide + MIME-spawned clients |
| Shell chrome architecture | 90 | 78 | **86** | i18n menus; a11y menu open; status refresh |
| FreeDesktop | 90 | 80 | **88** | OpenURI file://; nmcli; Inhibit→idle; **PW stubs** |
| A11y / i18n | 88 | 68 | **84** | DoAction + menu/dock/desktop menus live; Orca incomplete |
| Multi-monitor / HDR-VRR | 90 | 70 | **82** | Settings arrange + compositor layout parse |
| Polish / packaging / CI | 90 | 80 | **86** | Checklist + session_entry_smoke_report; unit suite |
| **Overall (equal-weight mean)** | **~90** | **~77** | **~85** | **(88+84+78+86+83+86+88+84+82+86)=845; 845/10=84.5 → ~85** |

### 13.3 What remains for honest ≥90

Nearly all domains **≥85 with live evidence**, including: greeter→session exercised;
client buffers routinely (not placeholder fallback); PipeWire ScreenCast or honest
“unavailable” UX; Orca activating core chrome including menus; display arrange
applied to outputs; window rules moving real surfaces. See deep audit § “What would
honest ≥90 require” and `WARPATH_SCORECARD.md` residual table.

### 13.4 Capability evidence (technical — warpath-verified)

| Criterion | Status | Evidence |
|---|---|---|
| Nested layer compose | **yes** | `main.rs` render under→windows→over |
| DRM present path | **yes (code)** | `session_drm` commit/page_flip |
| Workspace filter in paint | **yes (main)** | `windows_visible_for_paint` / `workspace_state.is_visible` |
| Per-window SHM vs placeholder | **yes (prefer SHM)** | `window_paint_source`; placeholder only if zero elements |
| `RETROSHELL_OUTPUTS_LAYOUT` | **yes (env bridge)** | shell `apply_display_plan_env` → compositor parse |
| Layer-shell chrome client | **yes (bind path)** | `layer_shell_client`; dual kit paint remains |
| Session power plans | **yes (wired spawn)** | `session_actions` + menu |
| DoAction → shell handlers | **partial live** | lock/log_out/force_quit/ws/window/dock/desktop/**menu open** + dock/desktop context status windows; Orca still incomplete |
| Portal D-Bus (subset) | **yes (plan-level extras)** | Secret/Print/Inhibit + Screenshot/…; ScreenCast stubs |
| Inhibit store → idle | **yes (in-process)** | `active_idle_inhibit_state` merged in shell `update` |
| Install session files | **yes (packaging)** | `scripts/install-session-files.sh` (+ dry-run) |
| Live greeter → session | **NOT RUN** | packaging/checklist only |
| Host unit tests | **yes** | shell/kit/compositor lib green (`verify_daily_driver_checklist`) |

### 13.5 Bottom line

- **Overall ~85** (mean **84.5**, sum **845**), not 90 or 100 — honest vs Plasma/GNOME daily driver.
- README / agents must not re-inflate to 90 without greeter + PW + Orca end-to-end + §12.
- §12 remains **0 / 7 fully met**.

*Skeptic deep audit 2026-07-11 — 90 claim rejected; warpath rescore; arithmetic fixed sum 845.*
