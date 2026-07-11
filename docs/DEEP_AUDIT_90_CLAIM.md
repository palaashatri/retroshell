# Deep audit of the ~90 claim (2026-07-11)

**Question:** Does RetroShell honestly earn **≥90 / 100** as a Plasma/GNOME-class
daily-driver laptop DE under the **original equal-weight methodology**?

**Answer: No.** The published **~90 (mean 89.6)** was **score theater**: pure policy
modules, unit tests, and bind-path scaffolding were scored as if they were live
Plasma-grade integration. This audit corrects that.

**Corrected overall after claim rejection: ~76 / 100** (mean **75.6**).  
**After immediate live-wiring follow-up (same day): ~77 / 100** (mean **77.2**).  
Still **not** 90 — do not re-inflate.

---

## Methodology (unchanged)

100 = replace Plasma for a week without constant workarounds.  
Domain scores reflect **workability**, not “lines of pure Rust that compile.”  
**Pure plan without live path ≠ domain 90.**

§12 “we made it” criteria remain the north star: **0 / 7 fully met** still.

---

## Critical evidence against ~90

| Claim that inflated score | Reality (verified this audit) |
|---|---|
| Compositor workspace composition filter → 90 | **Was** tests-only; **now wired** into nested `main.rs` paint + hit-test + Super+Left/Right/1–8 (still not Plasma). |
| Multi-monitor display arrange → 90 | Pure `plan_display_apply` + `DisplayConfig::plan_arrangement`. **Still no Settings UI apply / live modeset.** |
| i18n / a11y 88 | **Was** catalog-only; **now** lock screen uses `tr()`. Menus still mostly English; Orca still incomplete. |
| FreeDesktop Secret/Print/Inhibit → 90 | **Was** pure-only; **now** simplified Secret/Print/Inhibit on portal bus (still plan-level, not keyring/CUPS). ScreenCast still **placeholder node_id**. |
| Window rules → multi-client 90 | Rules apply skip-taskbar on FTL labels. **Workspace/maximize/float not applied to compositor surfaces.** |
| Own compositor 90 | Nested path real; DRM present code real; **client SHM may fall back to colored placeholder rects** (`main.rs` render). Not KWin-class. |
| Session 88 | Packaging + verify scripts **PASS**. **Live greeter → session NOT RUN.** Power actions spawn `systemctl` (often fails in lab/container). |
| Layer-shell chrome 90 | Bind path exists; **kit paint dual path still required** when unbound (`should_paint_kit_chrome`). |
| AT-SPI Text/Component on bus | Exported at **register-time snapshot**; DoAction **advisory**; extents often zero; **not Orca-complete**. |
| Host tests green | **True** (239 shell lib + kit + compositor). Tests prove pure correctness, **not** Plasma-week workability. |

Commands re-checked this audit:

```text
cargo test -p retro-shell -p retro-kit -p retro-compositor  → green
rg filter_visible crates/  → only lib.rs + tests (not main.rs)
rg portal_extra in portal_dbus → no matches
rg '\btr\(' in shell (excl i18n/tests) → no UI callers
./scripts/verify_greeter_session.sh → PASSED (packaging only; no live DM)
```

---

## Corrected scorecard

| Domain | Claimed (~90 card) | **Honest now** | Why |
|---|---:|---:|---|
| First-party productivity apps | 90 | **86** | Real suite + Force Quit; not the DE bottleneck |
| Toolkit / look & feel | 90 | **80** | Kit works; a11y structural; DoAction advisory |
| Session login / packaging | 88 | **74** | Install artifacts OK; live greeter **NOT RUN** |
| Own compositor as session WM | 90 | **76** | Compose/DRM code; placeholders; filter unused in main |
| Multi-client window management | 90 | **74** | Spawn + FTL; rules partial; shell-only workspace hide |
| Shell chrome architecture | 90 | **78** | Menus/lock/dock real; layer-shell dual path |
| FreeDesktop | 90 | **76** | Some portals on bus; Secret/Print/Inhibit pure; PW stubs |
| A11y / i18n | 88 | **62** | i18n unused in UI; Orca incomplete; bus snapshots |
| Multi-monitor / HDR-VRR | 90 | **70** | Env multi-output + pure arrange; no live KScreen apply |
| Polish / packaging / CI | 90 | **80** | Tests + Docker + docs; prior score inflation |
| **Overall (equal-weight mean)** | **~90** | **~76** (post-wire **~77**) | **Claim audit: 75.6 → 76**. After filter+portal+i18n lock wire: (86+80+74+80+76+78+80+68+70+80)/10 = **77.2 → 77** |

---

## What would honest ≥90 require (not a redefinition)

Nearly all domains **≥85 with live integration**, including:

1. Greeter → session exercised (or VM-logged evidence), not only desktop files.  
2. Compositor present path uses workspace visibility; clients show real buffers routinely.  
3. ScreenCast/PipeWire or honest “not available” UX — not stub node_ids scored as 90.  
4. Secret/Print/Inhibit on bus **or** dropped from FreeDesktop claims.  
5. i18n actually driving chrome strings; Orca can activate core chrome (DoAction → real handlers).  
6. Display arrange applied to outputs (or Settings apply path that works nested).  
7. Window rules moving real surfaces across workspaces.

**Honest ≥100** under this methodology would mean §12 **7/7** + week-long Plasma replace. That is multi-month DE work, not one more pure module.

---

## Bottom line

- **We do not deserve 90.** Corrected score **~76**.  
- Prior 90 claim is **withdrawn** as inflated integration credit.  
- Path forward: **wire pure modules into live paths**, re-score only after evidence, climb honestly toward 90 — never invent 100.

*Auditor stance: under-claim. Pure + unit test ≠ daily-driver domain 90.*
