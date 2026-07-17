# RetroShell — FULL AUDIT (honest) — 2026-07-11

**Question:** Is everything absolutely done at 100%?  
**Answer:** **No — not absolutely.** Production-critical code for the planned feature set is largely **implemented, tested on host, and runnable in Docker (labwc path)**. Several items are **environment-limited**, **partially integrated**, or **not release-shipped** (uncommitted). Below is evidence-only scoring.

Method: re-ran host tests, re-read code, inspected Docker image/runtime, visual screenshot, grep for stubs/lies.

---

## Executive scorecard

| Area | Grade | Honest note |
|---|---|---|
| Host unit/integration tests | **PASS** | 176 passed / 0 failed (this session) |
| Docker image build | **PASS** | Prior no-cache build exit 0; rebuild in progress for latest Settings fix |
| Docker desktop runtime | **PASS (labwc)** | Shell + noVNC + DE UI confirmed visually |
| Own compositor in Docker | **FAIL (env)** | DRI3 missing under Xvfb/Docker Desktop → dies; labwc fallback works |
| Own compositor on Pi/native GPU | **NOT RUN HERE** | Code + unit tests present; needs your Pi |
| Settings 8 themes + conf merge | **PASS** | 6/6 settings tests |
| Lock password gate | **PASS** | 2/2 tests; no "any key to unlock" |
| wl_data_device send | **PASS (code)** | Real fd write path; needs live multi-client QA |
| Multi-output | **PASS (code/tests)** | `RETROSHELL_OUTPUTS` parse + compositor wiring |
| XWayland | **PARTIAL** | Feature + spawn path; full paint path limited under nested X11 |
| HDR/VRR | **PASS (policy)** | Wired; nested reports `hdr_supported=false` honestly |
| AT-SPI | **PASS (export)** | Tree exported on bus; Embed may fail |
| NM / volume / battery / capture | **PASS (APIs + partial live)** | Modules + menus; volume/pactl proven in container |
| Settings → system volume | **PASS (code+test)** | Applied this audit (pactl/wpctl on slider save) |
| Visual QA gallery | **PARTIAL** | Real desktop PNG + short mp4; not 8 distinct UI states |
| Docs honesty | **PASS** | No PAM / any-key / "no longer Wayland client" lies found |
| Git release state | **NOT DONE** | Large uncommitted delta; not on origin |

**Bottom line:** Feature work for this pass is ~**90–95% complete in-tree**. Absolute 100% (compositor serving shell in *this* Docker, every UI state screenshoted, committed/pushed, Pi-verified GPU HDR/XWayland) is **not** true.

---

## Commands re-run this audit

### Host tests
```text
cargo test --workspace --exclude retro-compositor
→ passed=176 failed=0
```

### Compositor package tests (host-safe)
```text
cargo test -p retro-compositor
→ 17 + 17 tests, 0 failed
```

### Settings after volume bind
```text
cargo test -p settings → 6 passed; 0 failed
```

### Docker runtime (container `rs-qa`)
| Check | Result |
|---|---|
| Process `retro-shell` | alive |
| Process `labwc` | alive |
| Process `retro-compositor` | exits (DRI3) |
| noVNC | was HTTP 200 earlier |
| Display policy log | `hdr_supported=false` … then DRI3 error |
| AT-SPI log | Accessible tree exported, children=1 |
| Screenshot PNG | 1280×800 desktop with Finder/dock |
| ffmpeg 2s record | non-empty mp4 (~42KB) |
| pactl volume | set/get works |

---

## Feature-by-feature (plan DoD)

| DoD item | Status | Evidence |
|---|---|---|
| Host tests 0 failed | **YES** | 176/0 |
| docker build + noVNC + shell | **YES** | image + rs-qa |
| 8 themes + lock_password preserve | **YES** | settings tests |
| Lock gate | **YES** | tests; grep empty for any-key |
| data_device send | **YES (code)** | `send_selection` / `ServerDndGrabHandler::send` write bytes |
| Multi-output | **YES (code)** | `parse_outputs_spec` tests |
| XWayland path + package | **YES (code/pkg)** | Cargo feature + Dockerfile `xwayland` |
| HDR/VRR wired | **YES** | compositor policy log; DisplayRenderPolicy; Settings keys |
| AT-SPI tree on bus | **YES** | container logs |
| NM / Pulse / UPower fallbacks | **YES** | modules; pactl in Docker; NM Unavailable without daemon |
| Screenshot + record | **YES (tools path)** | menus + import/ffmpeg in container |
| Visual evidence | **PARTIAL** | prod desktop PNG + rec mp4 |
| Pi script | **YES** | `scripts/verify_pi.sh` |
| Docs reconciled | **YES** | stale-claim greps clean |

---

## Gaps that prevent "absolutely 100%"

### A. Environment (cannot fully close on this Mac Docker)
1. **`retro-compositor` cannot complete init without DRI3** on nested Xvfb/Docker Desktop.  
   - Fallback labwc is intentional and works.  
   - **Pi/native Linux** required to claim "our compositor serves the shell live."

### B. Integration polish (partially closed this audit)
2. **Settings volume** previously only wrote conf → now calls `pactl`/`wpctl` (host tests green; needs image rebuild to ship in container).  
3. **Settings Network** still primarily offline/dhcp preference; live line via `nmcli` when present (not full NM connect UI).  
4. **Shell internal windows** are still canvas rectangles, not separate Wayland surfaces (architecture truth).

### C. Depth / polish (honest remaining)
5. XWayland: spawn/handlers exist; **full GL composition of X11 clients** under nested X11 is limited.  
6. AT-SPI: object tree exported; **not Orca-complete** for every widget role/action.  
7. Selection send: implemented; **clipboard body** depends on filling mime maps / client sources.  
8. Visual gallery: not eight distinct app/settings/lock screenshots.  
9. **Uncommitted**: production work is not merged/pushed.

### D. Explicit non-goals still incomplete (README "still limited")
- Universal global menu for arbitrary external apps  
- HiDPI scale-factor tree  
- True multi-surface shell rewrite  

---

## What "100% for this product" should mean

| Definition | Met? |
|---|---|
| Plan DoD code items implemented + host tests green | **Mostly yes** |
| Docker DE demoable via noVNC | **Yes** |
| Own compositor live in Docker Desktop | **No** (DRI3) |
| Own compositor live on GPU Linux | **Unverified here** |
| Git clean, released | **No** |

---

## Recommended path to true 100%

1. Commit this branch; open PR.  
2. On Raspberry Pi: `./scripts/verify_pi.sh` then run compositor+shell; paste logs proving `retro-compositor is running` without labwc.  
3. Rebuild/push Docker image after Settings volume fix.  
4. Optional: scripted xdotool UI tour → 8 screenshots (Settings themes, lock, Terminal, etc.).  
5. Live DnD/clipboard test between two clients under labwc or compositor.

---

## Visual evidence on disk

- `docs/screenshots/prod-desktop.png` / `prod-01-desktop.png` — desktop + Finder  
- `docs/screenshots/prod-qa-rec.mp4` — short capture  

---

*Auditor stance: prefer under-claiming. Do not treat labwc fallback as "compositor 100%." Do not treat unit-tested modules as Pi-verified hardware paths.*
