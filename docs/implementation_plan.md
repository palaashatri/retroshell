# RetroShell — Path to KDE/GNOME-Class Polish & Workability

> **Purpose**: Roadmap for turning RetroShell from a working retro shell + first-party
> apps into a **daily-driver-class Linux desktop environment** (same *category* as
> Plasma / GNOME Shell — not a pixel clone). Written for implementers and agents:
> ordered phases, concrete modules, acceptance criteria, and honest dependencies.
>
> **Positioning**: Ambition is real. Full parity is multi-year. This document is the
> sequence of work that *actually gets there*, not marketing.
>
> **Related**: [`README.md`](../README.md) (ambition vs reality),
> [`ARCHITECTURE.md`](ARCHITECTURE.md), [`audit_2026-07-09.md`](audit_2026-07-09.md),
> [`FULL_AUDIT_2026-07-11.md`](FULL_AUDIT_2026-07-11.md).

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

## 2. Where we are today (baseline, 2026-07)

| Layer | Status | Gap vs DE-class |
|---|---|---|
| Toolkit (`retro-kit` / `retro-render` / `retro-sdk`) | Strong: widgets, wgpu, themes, menus | No scale-factor tree; limited a11y roles |
| First-party apps | Real I/O (Finder, Terminal, TextEdit, Settings, App Store) | Not all FreeDesktop portals; limited third-party |
| Shell chrome | Dock, menu bar, workspaces, notifications, password lock | Many “windows” still **paint-rects** inside one shell surface |
| Multi-client apps | `SessionClientRegistry` + spawn as processes under labwc/compositor | Shell does not yet *own* foreign surfaces; chrome dual-model |
| Compositor (`retro-compositor`) | Smithay nested-X11; selection send; multi-output env; XWayland path; HDR/VRR **policy** | Not DRM/KMS session; DRI3 fails under nested Docker; incomplete decoration/focus UX |
| System integration | NM status, volume CLI, UPower/`sys`, capture, AT-SPI export | No full connect UI, mixer, portals, greeter |
| Packaging | Docker + noVNC lab path | No distro session packages, no DM integration |

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

### Phase A — Session foundation
- [ ] A1 DRM/KMS compositor backend
- [ ] A2 Honest nested/software fallback
- [ ] A3 Session launcher / user unit
- [ ] A4 Display manager session entry
- [ ] A5 seatd/logind integration

### Phase B — Multi-client windowing
- [ ] B1 Full xdg_toplevel lifecycle under retro-compositor on hardware
- [ ] B2 Shell no longer fakes external app windows
- [ ] B3 Layer-shell bar/dock
- [ ] B4 Desktop surface
- [ ] B5 Foreign toplevel list
- [ ] B6 XWayland daily-driver quality
- [ ] B7 Popups/grabs correct

### Phase C — Shell chrome quality
- [ ] C1 External global menu story
- [ ] C2 Compositor-backed workspaces
- [ ] C3 App switcher / overview
- [ ] C4 FDO notification daemon
- [ ] C5 Compositor-level secure lock
- [ ] C6 HiDPI scale tree
- [ ] C7 Live theme propagation
- [ ] C8 Settings depth (NM/audio/displays)

### Phase D — FreeDesktop
- [ ] D1 xdg-desktop-portal
- [ ] D2 PipeWire screencast/audio depth
- [ ] D3 NetworkManager connect UI
- [ ] D4 Power/session inhibits
- [ ] D5 Polkit agent
- [ ] D6 Full .desktop/MIME
- [ ] D7 Clipboard manager
- [ ] D8 Input method support

### Phase E — A11y / i18n
- [ ] E1 Orca-usable AT-SPI
- [ ] E2 Keyboard-only
- [ ] E3 Contrast/motion prefs real
- [ ] E4 i18n
- [ ] E5 RTL basics

### Phase F — Polish / packaging
- [ ] F1 Performance budget on reference hardware
- [ ] F2 Startup budget
- [ ] F3 Crash recovery
- [ ] F4 Distro packages
- [ ] F5 Flatpak guidance
- [ ] F6 CI matrix including hardware
- [ ] F7 Security defaults

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

Until then: ship incrementally, stay honest, and follow the critical path above.
