# SLOPOS-I — Living Source of Truth

**Updated:** 2026-07-31 evening  
**This is the only living project doc.** Update *this file* (or a `qa/` evidence
file). Do **not** add new root-level session summaries, parallel roadmaps, or
split SoT markdown.

Supporting trees (not SoT — evidence / executors / history only):

| Path | Role |
|------|------|
| [`qa/`](qa/) | Screenshots + transcripts (stage / UI / Spotlight) |
| [`tasks/`](tasks/) | Atomic stage tasks with acceptance commands |
| [`specs/`](specs/) | Long-form design rationale (historical program design) |
| [`archive/`](archive/) | Superseded notes (including former PROGRAM/UI/HANDOFF/…) |
| [`../packaging/README.md`](../packaging/README.md) | Session packaging install notes |

```
TOC
1. Honesty contract
2. Snapshot & current focus
3. What this is / naming / architecture
4. Stages & definition of done
5. UI design (System 7)
6. Maturity vs GNOME/KDE — gaps & fix plan
7. VM ops (HANDOFF)
8. Gotchas & verify commands
9. Rules for agents
```

---

## 1. Honesty contract

Past failures we refuse to repeat: “~85/100 daily-driver” for a compositor that
never painted; “Spotlight complete” with **blank PNGs**; “theme 100%” in
archive notes. See [`archive/QA_REPORT_2026-07-26.md`](archive/QA_REPORT_2026-07-26.md).

> **A task is done only when its acceptance command passes.  
> A stage is done only when its QA doc passes on a real VM — evidenced by a
> screenshot or transcript, never by reading code or self-scoring.**

1. No unverified claims — say “unverified.”
2. Evidence or it didn’t happen — `qa/` only.
3. No fabricated work — don’t invent tasks to “confirm” fixed bugs.
4. Never claim GNOME/KDE parity or “daily driver” without evidence.
5. Prefer **removing fake UI** over adding decorative menus.

Blank / tiny PNGs (~hundreds of bytes) are **invalid** evidence.

---

## 2. Snapshot & current focus

| Item | Truth |
|------|--------|
| Product | **SLOPOS-I** (was RetroShell) |
| Crates / bins | `slopos-{render,kit,sdk,shell,bus,compositor}`; `slopos-shell`, `slopos-compositor`, `slopos-lock` |
| Env / config | `SLOPOS_*`, `~/.config/slopos-i` |
| Stages 0–3 | **VERIFIED** — `qa/stage-0.md` … `stage-3.md` |
| Stage 4 | Packaging authored; clean install / ISO DoD **unverified** — `qa/stage-4.md` |
| Spotlight + themed desktop | Visually proven on UTM — `qa/v0.2.0/` |
| UI vs System 7 kits | Improved; **not** kit-parity — `qa/ui-polish/` + §5 |
| vs GNOME / KDE | **~15–25%** daily-driver — research DE, not a peer — §6 |
| Defect H | `slopos-bus` still a facade |
| Defect J | Clicks proven Env B; **not** re-proven on UTM |
| Branch | Often `docs/program-design` — check `git status` (rename/docs may be uncommitted) |
| Host folder | May still be named `retroshell`; guest **`~/slopos-i`** |
| UTM SSH key | Still `~/.ssh/retroshell_utm` (legacy filename) |

### Current focus

| Priority | Work |
|----------|------|
| **Default** | UI polish (§5) — `slopos-sdk` paint → `qa/ui-polish/` → update §5 gaps |
| **Or Phase A** | Honesty: wire/remove fake menus; Stage 4 DoD; UTM Defect J (§6) |
| **If asked “real DE”** | Phases B→C (§6): bus, portals, DRM/XWayland, WM |
| **Never** | New session markdown; regress HDR/VRR; claim polish without screenshots |

Long-form rationale: [`specs/2026-07-30-slopos-i-de-program-design.md`](specs/2026-07-30-slopos-i-de-program-design.md).  
Spirit: [helloSystem](https://hellosystem.github.io/docs/).

---

## 3. What this is / naming / architecture

Classic-Mac-styled Linux **desktop environment** in Rust: own Wayland compositor
(smithay), shell, toolkit, SDK, five first-party apps.

**Paint reality:** many `slopos-kit` `Widget::draw` impls are **empty stubs**.
Pixels come from **`slopos-sdk::draw_widget`** walking the widget tree. Spotlight
only painted once widgets were on that path (`Panel` + children).

### Naming

| Kind | Form |
|------|------|
| Product | **SLOPOS-I** |
| System menu | **SLOPOS** |
| Config | `~/.config/slopos-i` |
| Env | `SLOPOS_*` |
| Crates / bins | `slopos-*` |
| Session | `slopos-i.desktop`, `start-slopos-i` |
| Chromeless desktop title | **`SLOPOS-I Desktop`** (special-cased in `draw_window` — do not rename casually) |

### Architecture

| Crate | Role |
|-------|------|
| `slopos-compositor` | Smithay compositor — Nested X11 + DRM/KMS; HDR/VRR hooks |
| `slopos-shell` | Menu, dock, workspaces, Spotlight, portals, lock, layer-shell |
| `slopos-sdk` | App framework + **primary paint path** |
| `slopos-kit` | Widgets (layout/input; many draw stubs) |
| `slopos-render` | wgpu / text plumbing |
| `slopos-bus` | Real in-process/queue IPC (`SloposBus`) — thread-safe transport queue & dispatch |
| `apps/*` | Finder, Settings, TextEdit, Terminal, App Store |

### Runtime gates / QA hooks

| Knob | Meaning |
|------|---------|
| `SLOPOS_LAYER_SHELL_CHROME=1` | Layer-shell desktop (menu Top / dock Bottom / wallpaper Background) |
| `SLOPOS_QA_SPOTLIGHT=<query>` | One-shot Spotlight for screenshots |
| `LIBGL_ALWAYS_SOFTWARE=1` + `GALLIUM_DRIVER=llvmpipe` | **Required on UTM** |
| `~/.config/slopos-i/settings.conf` | Theme, lock password, etc. |

### Known defects

| ID | Summary | Plan |
|----|---------|------|
| **H** | `slopos-bus` discards sends / never receives | Phase B (§6) |
| **J** | Toolkit clicks — Env B only; not re-proven UTM | Phase A (§6) |

---

## 4. Stages & definition of done

| Stage | Goal | Status | Evidence |
|------:|------|--------|----------|
| 0 | VM + KMS + workspace build | **VERIFIED** (Env B 2026-07-30) | `qa/stage-0.md` · `tasks/stage-0-vm-foundation.md` |
| 1 | App paints on compositor DRM | **VERIFIED** | `qa/stage-1.md` · `tasks/stage-1-prove-live-path.md` |
| 2 | Input, shortcuts, lock, clickable chrome | **VERIFIED** (Env B + layer re-QA) | `qa/stage-2.md` · `tasks/stage-2*.md` |
| 3 | `.app` store install → discover → launch | **VERIFIED** (Env A 2026-07-31) | `qa/stage-3.md` · `tasks/stage-3-app-bundles.md` |
| 4 | Layered install + ISO | **CODE-COMPLETE / DoD UNVERIFIED** | `qa/stage-4.md` · `tasks/stage-4-distribution.md` |

**Env note:** Recent UI / Spotlight / Stage 3 → **Env A (UTM Ubuntu aarch64)**.  
Stages 0–2 DRM proofs → **Env B (VBox Arch x86_64)**. Both valid (§7).

Stage 4 packaging (`install.sh`, PKGBUILD, `.deb`, archiso, verify scripts) is
in-tree. Tasks **4.5 / 4.6 / 4.8** have **no transcripts** yet.

### DoD (per stage)

- **0:** SSH `ls /dev/dri/card0` + `cargo build --release --workspace` in VM  
- **1:** Screenshot of Finder on `slopos-compositor` (not only labwc), or evidenced diagnosis  
- **2:** Lock not bypassable by app launch; password unlocks; `Super+O` → Finder  
- **3:** Store installs `.app`; appears in Finder/dock; launches  
- **4:** Login-selectable session on clean Arch **and** clean Ubuntu-server; ISO boots  

Task format for executors: [`tasks/README.md`](tasks/README.md).

### Recent changelog (evidence-backed)

- Stages 0–3 verified; Stage 4 authored only  
- Spotlight fixed (was invisible) — `qa/v0.2.0/`  
- UI chrome polish — `qa/ui-polish/`  
- Rename RetroShell → SLOPOS-I in-tree; UTM release build of compositor/shell/lock/finder/settings OK  
- Session packaging verify scripts PASS (`DesktopNames=SLOPOS-I`)

---

## 5. UI design (System 7)

**Status:** In progress — **not** kit-parity.  
**Evidence:** `qa/ui-polish/`, `qa/v0.2.0/`.  
**Paint:** `crates/slopos-sdk/src/lib.rs`. Kit draw stubs ≠ on-screen widgets.

### Goal

Match Classic Mac / System 7 from open kits and Figma — **without** Apple marks —
without ripping HDR/VRR / compositor roadmap.

### References

| Source | Use |
|--------|-----|
| [Calculable/System7Components](https://github.com/Calculable/System7Components) | Paint recipes |
| [Kelsidavis/System7](https://github.com/Kelsidavis/System7) | OS reimpl / behavior |
| [Figma Classic Mac UI Kit](https://www.figma.com/community/file/1392611044307310359/classic-macintosh-ui-kit) | Visual target |
| [Figma LGMlwNCoVdakZxDBvPKg1W](https://www.figma.com/design/LGMlwNCoVdakZxDBvPKg1W/Classic-Macintosh-UI-Kit--Community-?node-id=0-1) | Chrome detail |
| [Figma System 7–like](https://www.figma.com/design/8LqAFnsUxWQd4XeT6fPUEa/System-7--Apple-MacOS-7--like-UI-Kit--Community-?node-id=1-2) | Extra reference |

### Constraints

- SLOPOS glyph / **“SLOPOS”** menu — no rainbow Apple  
- Polish in kit/SDK canvas only  
- Fresh non-blank `qa/ui-polish/` screenshots required (UTM: sway+grim, §7)  
- Keep title **`SLOPOS-I Desktop`** special-case  

### Palette (light)

| Token | Hex |
|-------|-----|
| Background | `#FFFFFF` |
| Foreground | `#000000` |
| Gray100–500 | `#EFEFEF` … `#666666` |
| Lavender100 | `#DADAFC` (focused title rail) |

**Graphite:** full chrome must go dark (menu, windows, dock, icons) — not light
chrome on dark wallpaper. `theme=dark` in `settings.conf`.

### Port map

| System7Components | SLOPOS-I |
|-------------------|----------|
| `system73DBorder` | `draw_system7_3d_border` |
| `System73DButtonStyle` | multi-layer `draw_beveled_rect` |
| `System7Frame` header | `draw_classic_titlebar` |
| File/app symbols | fixed 32×32 `draw_labeled_icon` / `draw_*_icon` |
| Overlay panels | `Panel` + SDK `draw_widget` |

### UI gaps (update here when iterating)

1. Not kit-parity — Graphite menu/chrome still imperfect  
2. Icons schematic vs pixel art; Hard Disk/folder confusion possible  
3. Typography block/sans — not Chicago/Geneva  
4. Controls — checkbox/radio/slider/alert not ported  
5. Kit `draw()` stubs — SDK path required  
6. Broader DE gaps — §6  

### UI done recently (screenshots)

- Bevels, 3D border, title grips/close/zoom, lavender rail  
- Fixed 32×32 icons (no column bands)  
- Graphite helpers + desktop nameplates  
- Spotlight paints; Finder metadata for dirs  
- SLOPOS branding strings  

### How to iterate UI

1. Edit `slopos-sdk` (kit only if SDK will call it)  
2. Build on VM (§7); `SLOPOS_LAYER_SHELL_CHROME=1` + software GL on UTM  
3. `grim` → `qa/ui-polish/` (replace in place)  
4. Update **this section’s gap list** only  

### Standing HIG constraints

- One global menu bar (shell-owned)  
- Root-level dock (shell-owned)  
- Consistent metrics via kit/SDK  
- Trademark-safe SLOPOS / SLOPOS-I  

Also: Apple HIG / helloSystem UX / elementary HIG — classic-Mac spirit wins conflicts.

---

## 6. Maturity vs GNOME/KDE — gaps & fix plan

**Headline:** Credible vertical-slice / research DE. **Not** a GNOME/KDE peer.  
Rough daily-driver maturity: **~15–25%**. Fair peers: early helloSystem-class demos.

A full DE usually needs: session compositor for arbitrary clients; integrated shell;
OS control that drives the system; `xdg-desktop-portal` + PipeWire; PAM/logind/polkit;
usable suite or Flatpak path; proven clean install. SLOPOS-I has (1)–(2) in classic-Mac
shape, thin slices of the rest, packaging for install that is **not DoD-verified**.

### Scorecard

| Dimension | /10 | Notes |
|-----------|----:|-------|
| Session compositor | 5 | Protocols real; labwc fallback; DRM scanout/XWayland incomplete |
| Shell chrome | 5 | Menu/dock/Spotlight; layer-shell env-gated |
| Window management | 3 | Weak move/resize; workspaces unsynced shell↔compositor |
| Lock / security | 3 | No PAM — conf/env password |
| Portals / 3rd-party | 2 | Custom portal; FileChooser/ScreenCast stubs |
| Settings (OS control) | 3 | Conf editor; many keys apply at startup only |
| Finder / TextEdit / Terminal / Store | 3–4 | Usable cores; menus/ecosystem thin |
| Suite breadth | 2 | Five apps; no browser/mail/media |
| Visual polish | 4 | §5 |
| Install / ship | 2 | Stage 4 DoD pending |
| IPC | 1 | Defect H |
| **Daily-driver overall** | **~2** | VM demo / dogfood — not general computing |

### Gap register

**Compositor:** C1 DRM scanout incomplete · C2 no XWayland on DRM · C3 session-lock DRM-only · C4 weak WM · C5 workspace desync · C6 labwc fallback  

**Shell:** S1 decorative menus · S2 write-only settings · S3 thin NM/volume/power · S4 no PAM · S5 portal subset · S6 Spotlight no file index · S7 no PipeWire screencast  

**Toolkit:** T1 kit draw stubs · T2 missing controls · T3 in-process clipboard · T4 Defect J · T5 fonts  

**Apps:** A1 Finder depth · A2 Settings depth · A3 TextEdit · A4 Terminal · A5 Store · A6 no suite  

**Platform:** P1 Defect H · P2 not standard portal bus · P3 Stage 4 DoD · P4 rename leftovers (`RetroBus`, host paths)  

### Fix phases

Do not mark a phase done without `qa/` evidence. UI (§5 / Phase E) may run parallel
to A/B without regressing HDR/VRR.

#### Phase A — honesty / ship *(near-term)*

| Work | Closes |
|------|--------|
| Wire or remove decorative menus | S1, A1–A4 |
| UTM Defect J re-proof | T4 |
| Stage 4 clean-VM / greeter DoD | P3 |
| Rename leftovers (`RetroBus` → `SloposBus`) | P4 (**VERIFIED**) |

#### Phase B — integration spine

| Work | Closes |
|------|--------|
| Real `slopos-bus` + theme notify | P1 / H |
| Live settings apply | S2 |
| System clipboard + primary | T3 |
| Sync shell ↔ compositor workspaces | C5 |
| Standard `xdg-desktop-portal` path | P2, S5 |

#### Phase C — compositor / session

| Work | Closes |
|------|--------|
| DRM client buffer presentation | C1 |
| XWayland on DRM | C2 |
| Interactive move/resize/maximize | C4 |
| PAM lock | S4 |
| Tighten compositor vs labwc default | C6 |
| PipeWire screencast | S7 |

#### Phase D — app suite

Finder list/open-with/rename · Settings NM + modeset · TextEdit portal+clipboard ·
Terminal menus/tabs · Store remove/update/HTTP · more utilities **or** Flatpak escape hatch  

#### Phase E — visual kit parity *(parallel)*

Fonts · System7Components controls · pixel icons · Graphite fidelity · Spotlight file search  
(Detail: §5)

### Agent priority

| User asks… | Do |
|------------|-----|
| Default / continue | Phase E UI **or** Phase A honesty |
| “Make it a real DE” | B then C |
| “Ship / install” | Phase A Stage 4 first |
| “More apps” | D after B — or accept thin demos |

### Do not regress

Smithay core protocols · layer-shell chrome · Spotlight paint + QA hook · Stages 0–3
behaviors · `.app` pipeline · HDR/VRR hooks on DRM  

---

## 7. VM ops (HANDOFF)

All graphical/live smithay work is **on the Linux VM**, never as a full session on
macOS/Windows hosts.

| Aspect | **A: macOS + UTM** (recent UI / Stage 3) | **B: Windows + VirtualBox** (Stages 0–2 DRM) |
|--------|------------------------------------------|-----------------------------------------------|
| Guest | Ubuntu 26.04 aarch64 (`Ubuntu`) | Arch x86_64 (`slopos-i-arch`) |
| GPU | virtio-gpu → `/dev/dri/card0` | VMSVGA+3D → `vmwgfx` |
| SSH | `~/.ssh/retroshell_utm`, `192.168.64.15:22` | `packaging/vm/qa_key`, `127.0.0.1:2222` |
| User | `ubuntu` | `retro` |
| Tree | `~/slopos-i` | confirm path |
| Screenshot | **sway + grim** (SIGUSR1 dump **BLOCKED**) | `VBoxManage screenshotpng` |
| Software GL | **Required** llvmpipe | usually not |

Ignore Mac-only Arch arm64 scripts (`arch-install-arm64.sh`, `provision-arm64.sh`)
for the current UTM Ubuntu guest.

### Env A — build loop

```bash
rsync -az --exclude target --exclude target-docker --exclude .git \
  --exclude 'docs/qa/**/*.png' --exclude docs/screenshots \
  -e "ssh -i ~/.ssh/retroshell_utm" \
  ./ ubuntu@192.168.64.15:/home/ubuntu/slopos-i/

ssh -i ~/.ssh/retroshell_utm ubuntu@192.168.64.15 \
  'cd ~/slopos-i && source ~/.cargo/env && cargo build --release -p <crate>'
```

**Disk:** ~30G LVM fills fast — never rsync `target*`. Clear guest `target*` if `df` ≈ 100%.

### Env A — DRM compositor

```bash
export XDG_RUNTIME_DIR=/run/user/1000 LIBSEAT_BACKEND=seatd \
       LIBGL_ALWAYS_SOFTWARE=1 GALLIUM_DRIVER=llvmpipe \
       SLOPOS_LAYER_SHELL_CHROME=1
./target/release/slopos-compositor
```

### Env A — screenshots (sway + grim)

```bash
export XDG_RUNTIME_DIR=/run/user/$(id -u)
export WLR_BACKENDS=headless WLR_LIBINPUT_NO_DEVICES=1
export LIBGL_ALWAYS_SOFTWARE=1 GALLIUM_DRIVER=llvmpipe
export SLOPOS_LAYER_SHELL_CHROME=1
# sway: output * { resolution 1280x800 }
sway -c /tmp/sway-headless.conf &
export SWAYSOCK=... WAYLAND_DISPLAY=wayland-1
./target/release/slopos-shell &
# optional: SLOPOS_QA_SPOTLIGHT=vol ./target/release/slopos-shell
sleep 7
grim docs/qa/ui-polish/01-desktop.png
```

Spotlight recipe detail: `qa/v0.2.0/QA-RESULTS.md`.

### Env B — notes

`packaging/vm/create-vm.ps1` → VMSVGA+3D. Sync via `qa_key` / port 2222.  
Screenshots: `VBoxManage controlvm slopos-i-arch screenshotpng out.png`
(`packaging/vm/qa-live.sh`) — real compositor scanout.

### Layer-shell

`SLOPOS_LAYER_SHELL_CHROME=1` → Background wallpaper/icons, Top menu, Bottom dock,
Overlay `slopos-i-menu-popup`. Unset → winit xdg-toplevel chrome (keep for apps).

---

## 8. Gotchas & verify commands

- Blank screenshots lie — reject tiny/empty PNGs  
- Kit `draw()` stubs ≠ pixels — follow SDK tree  
- Dependency Cargo.toml changes → long VM rebuilds  
- Keep winit app path working  
- Some `qa/*.md` may be CRLF (Windows)  
- Don’t “fix” Rust idents into invalid `SLOPOS-I` tokens (`SloposI` type is fine)  
- Product strings: SLOPOS-I / SLOPOS  

```bash
./scripts/verify_session_packaging.sh
./scripts/verify_greeter_session.sh

cd ~/slopos-i && source ~/.cargo/env
cargo build --release -p slopos-compositor -p slopos-shell -p finder
cargo test -p slopos-kit -p slopos-sdk -p slopos-shell --lib --release
```

Quick start:

```bash
cargo build --release -p slopos-compositor -p slopos-shell
SLOPOS_LAYER_SHELL_CHROME=1 ./scripts/start-slopos-i
# UTM also: LIBGL_ALWAYS_SOFTWARE=1 GALLIUM_DRIVER=llvmpipe
```

---

## 9. Rules for agents

1. **Read this file** first. Then `qa/` for evidence. Then `tasks/` if executing a stage task.  
2. **Edit this file** for status/gaps/plans — not new `docs/*.md` SoT files.  
3. UI claims → §5 + `qa/ui-polish/` (Spotlight bar also `qa/v0.2.0/`).  
4. DE gap claims → §6.  
5. One task ≈ one commit when using `tasks/`; no architecture freelancing.  
6. Commit/push only when the user asks.  

### Revision log

| Date | Change |
|------|--------|
| 2026-07-31 | Consolidated PROGRAM + UI + HANDOFF + FUTURE + MATURITY + docs README into this single living doc |
| 2026-07-31 | Renamed RetroBus to SloposBus (P4); implemented LocalTransport thread-safe queue & unit tests (Defect H) |
| 2026-07-31 | Created assets/slopos-logo.png, added Material/Retro icon style toggle, expanded missing icon kinds, fixed text alignment via Canvas::measure_text |
