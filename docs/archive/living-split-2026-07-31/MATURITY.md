# SLOPOS-I — Desktop maturity, gaps, and fix plan

> **Living SoT** for “how close are we to a real DE (GNOME/KDE class)?”  
> Visual language stays in [UI.md](UI.md). Stage status stays in [PROGRAM.md](PROGRAM.md).  
> Ops stay in [HANDOFF.md](HANDOFF.md). Do **not** mark rows done without VM evidence.

**Last audit:** 2026-07-31  
**Honest headline:** SLOPOS-I is a **credible vertical-slice / research DE**
(compositor + shell + five first-party apps). It is **not** a GNOME/KDE peer.
Rough daily-driver maturity: **~15–25%**. Fair peers: early helloSystem-class
demos, not Plasma 6 / GNOME 46.

---

## 1. What “full-fledged DE” means here

A shipping Linux DE (GNOME, KDE Plasma, Cosmic, etc.) typically provides:

1. A **session compositor** that hosts arbitrary Wayland (and usually XWayland) clients
2. **Shell chrome** (panels, launcher, notifications) integrated with that compositor
3. **OS control** (displays, network, power, users, updates) that actually drives the system
4. **xdg-desktop-portal** + PipeWire so third-party GTK/Qt/Flatpak apps work
5. **Secure session** (PAM lock, logind, polkit)
6. A **usable app suite** or a clear path to an ecosystem (Flatpak/deb)
7. A **proven install path** on clean machines

SLOPOS-I has (1) and (2) in a classic-Mac shape, thin slices of (3)–(6), and
packaging for (7) that is **not yet DoD-verified**.

---

## 2. Scorecard vs GNOME / KDE (honest)

| Dimension | Score (0–10) | Status | Evidence / notes |
|-----------|-------------:|--------|------------------|
| Session compositor | 5 | PARTIAL | Smithay: xdg-shell, layer-shell, DRM/libinput; labwc fallback; DRM scanout/XWayland incomplete |
| Shell chrome | 5 | PARTIAL | Menu, dock, desktop, Spotlight (UTM screenshots); env-gated layer-shell |
| Window management | 3 | PARTIAL | Focus/cascade/workspaces; weak move/resize; shell ↔ compositor workspaces **unsynced** |
| Lock / session security | 3 | PARTIAL | Lock exists; password = conf/env string — **no PAM** |
| Portals / 3rd-party apps | 2 | STUB/PARTIAL | Custom `org.slopos-i.Portal`; FileChooser/ScreenCast stubs; not standard portal bus |
| System Settings (OS control) | 3 | PARTIAL | Conf editor + some `pactl`/`nmcli`; many keys apply only at startup |
| File manager | 4 | PARTIAL | Browse/trash/drag; most menus decorative; no open-with / list view |
| Text editor | 4 | PARTIAL | Plain text + undo/find; no system clipboard / real file picker |
| Terminal | 4 | PARTIAL | PTY + VT + tabs; menus largely unwired |
| App Store | 3 | PARTIAL | Local SHA-256 `.app` install; no remote catalog / remove / Flatpak |
| Bundled suite breadth | 2 | MISSING | Five apps only — no browser, mail, media, utilities |
| Visual polish | 4 | PARTIAL | Improved System 7 chrome; not kit-parity ([UI.md](UI.md)) |
| Install / ship | 2 | UNVERIFIED | `install.sh`/PKGBUILD/ISO authored; Stage 4 DoD pending |
| IPC / integration | 1 | STUB | **Defect H** — `slopos-bus` discards sends |
| **Daily-driver overall** | **~2** | — | VM demo / stack dogfood — not general computing |

---

## 3. Gap register (what’s wrong)

### 3.1 Compositor / session

| ID | Gap | Impact |
|----|-----|--------|
| C1 | DRM path: client GL/SHM scanout incomplete; dumb-buffer present fallback | Foreign clients may not composite correctly on real seats |
| C2 | XWayland not wired on DRM session | Classic X11 apps fail on the production path |
| C3 | Session-lock only on DRM compositor path | Nested/dev path cannot host `slopos-lock` protocol |
| C4 | Weak interactive WM (move/resize/maximize for generic clients) | Feels unfinished vs Mutter/KWin |
| C5 | Shell `WorkspaceManager` ≠ compositor `WorkspaceState` | Workspace UI and Super+keys diverge |
| C6 | Labwc fallback still in `start-slopos-i` | Session may silently not be “our” compositor |

### 3.2 Shell / desktop services

| ID | Gap | Impact |
|----|-----|--------|
| S1 | Many global / app menu items decorative (no handler) | UI lies about capabilities |
| S2 | Settings mostly write-only conf; weak live apply | Not an OS control center |
| S3 | Network/volume/power are status + CLI helpers | No full NM / PipeWire session UI |
| S4 | Lock without PAM | Not deployable as a secure lock |
| S5 | Notifications / portals are subsets | Third-party apps expect FreeDesktop portals |
| S6 | Spotlight: apps + hardcoded settings; no file index | Far from KRunner / GNOME Overview |
| S7 | Capture via X11 `import`/`ffmpeg` recipes | No PipeWire Wayland screencast |

### 3.3 Toolkit / SDK

| ID | Gap | Impact |
|----|-----|--------|
| T1 | Most kit `Widget::draw` are stubs; SDK paints | Easy to “implement” widgets that never show |
| T2 | Missing controls (checkbox, radio, alert, …) | Apps can’t match System 7 kits |
| T3 | In-process clipboard only | No system / primary selection |
| T4 | **Defect J** — clicks not re-proven on UTM | Interaction confidence incomplete on Env A |
| T5 | Typography is block/sans, not Chicago/Geneva-class | Visual gap vs kits ([UI.md](UI.md)) |

### 3.4 Apps

| ID | Gap | Impact |
|----|-----|--------|
| A1 | Finder: no list view, open-with, rename UI, search, wired View/Edit menus | Not a daily file manager |
| A2 | Settings: no users, Bluetooth, printers, updates, live display modeset | Conf editor, not GNOME Settings |
| A3 | TextEdit: no system clipboard, file dialog, syntax, tabs | Demo editor |
| A4 | Terminal: menus/zoom/copy mostly unwired; thin profiles | Demo terminal |
| A5 | App Store: no HTTP catalog, remove/update, signing beyond sha256 | Half a store |
| A6 | No browser / mail / media / utilities | Suite incomplete by definition |

### 3.5 Platform / ship

| ID | Gap | Impact |
|----|-----|--------|
| P1 | **Defect H** — `slopos-bus` facade | Blocks cross-process theme/events |
| P2 | Not on standard `xdg-desktop-portal` bus | GTK/Qt/Flatpak integration blocked |
| P3 | Stage 4 clean Arch/Ubuntu/ISO DoD unverified | Cannot claim “installable DE” |
| P4 | Rename leftovers (`RetroBus`, host path/key names) | Confusion for contributors |

---

## 4. Fix plan (phased)

Phases are **ordered by leverage toward a real DE**. UI polish ([UI.md](UI.md))
can run in parallel with Phase A/B **without** regressing HDR/VRR.

Do not declare a phase done without `qa/` evidence.

### Phase A — Stop lying / make the slice honest *(near-term)*

| Work | Closes | Acceptance idea |
|------|--------|-----------------|
| Wire or remove decorative menus (shell + apps) | S1, A1–A4 | Menu items either invoke real actions or are absent; screenshot + smoke |
| Document + QA: kit stubs must paint via SDK | T1 | [UI.md](UI.md) + HANDOFF already; keep enforcing |
| UTM Defect J re-proof (`wtype`/`ydotool` or harness) | T4 | Click transcript on Env A in `qa/` |
| Stage 4 clean-VM / greeter DoD | P3 | Fill `qa/stage-4.md` 4.5/4.6 (+ 4.8 when ready) |
| Finish rename leftovers (`RetroBus` → `SloposBus`, etc.) | P4 | Compile + grep clean |

### Phase B — Integration spine *(makes everything else compound)*

| Work | Closes | Acceptance idea |
|------|--------|-----------------|
| Repair **Defect H** — real `slopos-bus` transport | P1 | Send/receive round-trip test + one live consumer (e.g. theme notify) |
| Live settings apply (file watch and/or bus) | S2 | Toggle theme/dock without full session restart; screenshot |
| System clipboard + primary selection in kit/SDK | T3 | TextEdit cut/paste with external client |
| Sync shell ↔ compositor workspaces | C5 | Super+N and menu switch same workspace for foreign clients |
| Standard portal path (implement or proxy `xdg-desktop-portal`) | P2, S5 | GTK file chooser / screenshot from a third-party app |

### Phase C — Compositor / session competence

| Work | Closes | Acceptance idea |
|------|--------|-----------------|
| DRM client buffer presentation / scanout | C1 | Foreign wl client visible on DRM seat screenshot |
| XWayland on DRM path | C2 | `xterm`/`xeyes` on DRM session |
| Interactive move/resize/maximize for xdg clients | C4 | Pointer drag chrome; screenshot |
| PAM (or pam-adjacent) lock | S4 | Unlock with real user password |
| Prefer own compositor; labwc only explicit fallback | C6 | `start-slopos-i` honesty already; tighten defaults |
| PipeWire screencast via portal | S7 | Portal ScreenCast returns real node; OBS/Firefox smoke |

### Phase D — App suite credibility

| Work | Closes | Acceptance idea |
|------|--------|-----------------|
| Finder: list view, open-with, rename, wire View/Edit | A1 | Stage-style QA screenshots |
| Settings: NM Wi-Fi UI, display modeset that hits compositor | A2, S3 | Connect Wi-Fi + change resolution on VM |
| TextEdit: portal file chooser + system clipboard | A3 | Open/save via portal |
| Terminal: wire copy/paste/zoom; visible tab bar | A4 | Manual QA checklist |
| App Store: remove/update + optional HTTP catalog | A5 | Install → remove → rescan |
| Either ship more first-party utilities **or** Flatpak escape hatch | A6 | Documented path to run Firefox |

### Phase E — Visual kit parity *(ongoing, parallel)*

Tracked in detail in [UI.md](UI.md):

- Chicago/Geneva-class fonts (T5)
- System7Components controls (T2)
- Pixel icons, Graphite fidelity
- Spotlight file search + icons (S6)

---

## 5. Priority guidance for agents

| If the user asks for… | Do this |
|-----------------------|---------|
| Default / “continue” | Phase E UI polish **or** Phase A honesty fixes — not new session markdown |
| “Make it a real DE” | Phase B then C (bus, portals, DRM/XWayland, WM) |
| “Ship / install” | Phase A Stage 4 DoD first |
| “More apps” | Phase D only after B (clipboard/portals) or accept thin demos |

**Standing rule:** never claim GNOME/KDE parity, “daily driver,” or Stage 4 done
without evidence. Prefer removing fake UI over adding more façades.

---

## 6. What is already real (do not regress)

Keep these working while closing gaps:

- Smithay compositor protocols (xdg-shell, layer-shell, foreign-toplevel, clipboard)
- Layer-shell exclusive chrome (`SLOPOS_LAYER_SHELL_CHROME=1`)
- Spotlight paint path + `SLOPOS_QA_SPOTLIGHT` QA hook
- Stages 0–3 verified behaviors (see `qa/stage-*.md`)
- `.app` bundle scan / install pipeline
- HDR/VRR hooks on DRM path (even if incomplete end-to-end)

---

## 7. Revision

| Date | Change |
|------|--------|
| 2026-07-31 | Initial maturity audit + phased fix plan after GNOME/KDE comparison |
