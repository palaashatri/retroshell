# Deep audit of the ~90 claim (2026-07-11)

**Question:** Does RetroShell honestly earn **≥90 / 100** as a Plasma/GNOME-class
daily-driver laptop DE under the **original equal-weight methodology**?

**Answer: No.** The published **~90 (mean 89.6)** was **score theater**: pure policy
modules, unit tests, and bind-path scaffolding were scored as if they were live
Plasma-grade integration. This audit corrects that.

**Corrected overall after claim rejection: ~76 / 100** (mean **75.6**).  
**After immediate live-wiring follow-up (same day): ~77 / 100** (mean **77.2**).  
**After warpath live integration (same day): ~85 / 100** (mean **84.6**).  
Still **not** 90 — do not re-inflate. See `GOAL_DEEP_AUDIT_FINAL.md`. Full arithmetic: **`docs/WARPATH_SCORECARD.md`**.

---

## Methodology (unchanged)

100 = replace Plasma for a week without constant workarounds.  
Domain scores reflect **workability**, not “lines of pure Rust that compile.”  
**Pure plan without live path ≠ domain 90.**

§12 “we made it” criteria remain the north star: **0 / 7 fully met** still.

---

## Critical evidence against ~90

| Claim that inflated score | Reality (verified this audit; warpath notes) |
|---|---|
| Compositor workspace composition filter → 90 | **Now wired** into nested `main.rs` paint + hit-test + Super+Left/Right/1–8 (still not Plasma). |
| Multi-monitor display arrange → 90 | Shell **EmitLayoutEnv** + compositor **RETROSHELL_OUTPUTS_LAYOUT** parse. **Still no Settings UI modeset / live KMS apply.** |
| i18n / a11y 88 | Lock + **system menus** use `tr()`. Orca incomplete; `chrome.menu.activate` **log-only**. |
| FreeDesktop Secret/Print/Inhibit → 90 | **On portal bus** (plan-level, not keyring/CUPS). Inhibit store → shell idle. ScreenCast still **placeholder node_id**. |
| Window rules → multi-client 90 | Rules apply skip-taskbar on FTL labels. **Workspace/maximize/float not fully applied to compositor surfaces.** |
| Own compositor 90 | Nested path real; DRM present code real; **prefer SHM**, placeholder only if no committed buffer. Not KWin-class. |
| Session 88 | Packaging + `install-session-files.sh` + daily checklist **PASS**. **Live greeter → session NOT RUN.** |
| Layer-shell chrome 90 | Bind path exists; **kit paint dual path still required** when unbound (`should_paint_kit_chrome`). |
| AT-SPI Text/Component on bus | Exported; DoAction **queue drains** for lock/log_out/force_quit/ws/window/dock/desktop; extents often zero; **not Orca-complete**. |
| Host tests green | **True** (shell lib + kit + compositor). Tests prove pure correctness, **not** Plasma-week workability. |

Commands re-checked at claim rejection (and warpath residuals re-checked at rescore):

```text
cargo test -p retro-shell -p retro-kit -p retro-compositor  → green (lib paths)
./scripts/verify_daily_driver_checklist.sh → PASSED (packaging + units only)
./scripts/verify_greeter_session.sh → PASSED (packaging only; no live DM)
# residuals still true:
# live greeter NOT RUN; PipeWire streams stubs; menu.activate not live; §12 0/7
```

---

## Corrected scorecard (claim audit → first wire)

| Domain | Claimed (~90 card) | **Honest after claim audit** | Why (at rejection) |
|---|---:|---:|---|
| First-party productivity apps | 90 | **86** | Real suite + Force Quit; not the DE bottleneck |
| Toolkit / look & feel | 90 | **80** | Kit works; a11y structural; DoAction then advisory |
| Session login / packaging | 88 | **74** | Install artifacts OK; live greeter **NOT RUN** |
| Own compositor as session WM | 90 | **76→80** | Filter then wired in main; placeholders remain |
| Multi-client window management | 90 | **74→76** | Spawn + FTL; rules partial |
| Shell chrome architecture | 90 | **78** | Menus/lock/dock real; layer-shell dual path |
| FreeDesktop | 90 | **76→80** | Secret/Print/Inhibit then on bus; PW stubs |
| A11y / i18n | 88 | **62→68** | Lock `tr()`; Orca incomplete |
| Multi-monitor / HDR-VRR | 90 | **70** | Env multi-output + pure arrange |
| Polish / packaging / CI | 90 | **80** | Tests + Docker + docs |
| **Overall (equal-weight mean)** | **~90** | **~76 → ~77** | Claim **75.6**; first wire **77.2** |

---

## Warpath progress (same day, after claim rejection)

Warpath closed **live paths** that the claim audit marked missing or pure-only.
This is **not** a redefinition of 100 and **not** a return to 90.

| Domain | Post-claim (~77) | **Warpath** | What landed (evidence) |
|---|---:|---:|---|
| First-party productivity apps | 86 | **86** | No DE-bottleneck change |
| Toolkit / look & feel | 80 | **83** | Kit DoAction pending → shell drain |
| Session login / packaging | 74 | **78** | `install-session-files.sh` + `verify_daily_driver_checklist.sh`; greeter still **NOT RUN** |
| Own compositor as session WM | 80 | **84** | Workspace paint/focus live; per-window SHM prefer |
| Multi-client window management | 76 | **81** | Live workspace hide/focus for clients; rules still partial |
| Shell chrome architecture | 78 | **83** | i18n menus; chrome a11y dispatch; dual path remains |
| FreeDesktop | 80 | **84** | Inhibit store → idle policy; Secret/Print plan-level; PW stubs |
| A11y / i18n | 68 | **82** | Menus + lock `tr()`; DoAction live set + menu.activate live |
| Multi-monitor / HDR-VRR | 70 | **76** | `RETROSHELL_OUTPUTS_LAYOUT` shell→compositor; no live modeset UI |
| Polish / packaging / CI | 80 | **84** | install-session + daily checklist + unit green |
| **Overall** | **~77** | **~85** | **(88+84+78+86+83+86+88+82+82+86)/10 = 84.6 → 85** |

### Still not closed (caps on 90)

1. Live greeter → session **NOT RUN** (packaging only).  
2. PipeWire ScreenCast streams still **stubs** (`node_id` placeholders).  
3. `chrome.menu.activate` may remain **partial / log-only**.  
4. §12 still **0 / 7 fully met**.  
5. Placeholder rects still possible when clients have no committed buffer.  
6. Display arrange is env-bridge, not live KMS/Settings apply.

See **`docs/WARPATH_SCORECARD.md`** for full arithmetic and residual table.

---

## What would honest ≥90 require (not a redefinition)

Nearly all domains **≥85 with live integration**, including:

1. Greeter → session exercised (or VM-logged evidence), not only desktop files.  
2. Compositor present path uses workspace visibility; clients show real buffers routinely.  
3. ScreenCast/PipeWire or honest “not available” UX — not stub node_ids scored as 90.  
4. Secret/Print backed by real services **or** dropped from FreeDesktop claims.  
5. i18n driving chrome strings; Orca can activate core chrome including menus.  
6. Display arrange applied to outputs (or Settings apply path that works nested).  
7. Window rules moving real surfaces across workspaces.

**Honest ≥100** under this methodology would mean §12 **7/7** + week-long Plasma replace. That is multi-month DE work, not one more pure module.

---

## Bottom line

- **We do not deserve 90.** Claim audit **~76**; first wire **~77**; warpath **~85**.  
- Prior 90 claim is **withdrawn** as inflated integration credit.  
- Path forward: keep wiring live paths, re-score only after evidence, climb honestly toward 90 — never invent 100.

*Auditor stance: under-claim. Pure + unit test ≠ daily-driver domain 90.*
