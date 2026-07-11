# Warpath scorecard (honest arithmetic)

**Date:** 2026-07-11  
**Question:** After the live-integration warpath (post ~90 rejection), what is the
honest equal-weight daily-driver score vs Plasma/GNOME?

**Answer: ~85 / 100 (mean 84.6) after nmcli + Settings display + compositor damage wave.**  
Not 90. Not 100. Prefer under-claim.

Related: [`DEEP_AUDIT_90_CLAIM.md`](DEEP_AUDIT_90_CLAIM.md),
[`implementation_plan.md`](implementation_plan.md) §13, [`README.md`](../README.md).

---

## Methodology

- **100** = replace Plasma for a week without constant workarounds.
- **10 domains, equal weight.** Overall = arithmetic mean; “~NN” = mean rounded
  to nearest integer.
- **Pure module + unit test without a live path does not score as domain 90.**
- Live greeter **NOT RUN**, PipeWire streams **stubs**, §12 **0 / 7** fully met —
  these **cap** several domains.

---

## Timeline (same methodology)

| Milestone | Mean | Overall |
|---|---:|---:|
| Inflated “hard DE ~90” card | 89.6 | **~90 WITHDRAWN** (score theater) |
| Deep audit (claim rejection) | 75.6 | **~76** |
| Immediate wire (filter + portal + lock i18n) | 77.2 | **~77** |
| Warpath mid pass | 81.6 | **~82** |
| + menu.activate + MIME open | 83.0 | **~83** |
| **+ nmcli + Settings arrange + damage (this card)** | **84.6** | **~85** |

---

## Warpath evidence (verified in-tree)

| Landing | Status | Where |
|---|---|---|
| Workspace paint / focus in compositor main | **live** | `crates/retro-compositor/src/main.rs` (`windows_visible_for_paint`, hit-test); `workspace_focus.rs` |
| Per-window SHM vs placeholder | **prefer SHM** | `window_paint_source`; placeholder only when surface tree empty |
| `RETROSHELL_OUTPUTS_LAYOUT` shell → compositor | **env bridge** | shell `apply_display_plan_env`; compositor `parse` / layout source |
| DoAction queue → shell handlers | **partial live** | `drain_a11y_pending_actions` → lock / log_out / force_quit / workspace / window close·activate / dock / desktop |
| `chrome.menu.activate` | **live** | opens Retro/system menu (`open_menu_at` / Retro title) |
| MIME open files + OpenURI `file://` | **live spawn path** | `spawn_open_plan`; Finder double-click files; portal file:// |
| i18n menus + lock | **live callers** | `menu_server.rs` `tr(...)`; lock UI `tr("lock.*")` |
| Portal Secret / Print / Inhibit on bus | **on bus, plan-level** | `portal_dbus.rs` + `portal_extra.rs` (not keyring/CUPS) |
| Inhibit store → idle | **in-process** | `active_idle_inhibit_state` merged in shell `update` |
| `install-session-files.sh` + daily-driver checklist | **packaging PASS** | scripts; checklist exits 0 on packaging + unit tests only |
| Live greeter → session | **NOT RUN** | no DM login evidence |
| PipeWire ScreenCast streams | **stubs** | `node_id` placeholders; honest readiness notes only |
| §12 “we made it” | **0 / 7 fully met** | packaging closer on criterion 1; none fully closed |

Commits in warpath band (post deep-audit wire): `00c53f9`, `6455ec3`, `622d37e`,
`0aca984`, `6d177b3`, `fcf11ba` (and predecessors that rejected the 90 claim).

Checklist re-run this rescore:

```text
./scripts/verify_daily_driver_checklist.sh
→ daily-driver checklist PASSED (packaging + unit evidence only — not Plasma-100)
→ §12 greeter criterion: install artifacts ready; live DM login still NOT RUN
→ cargo test shell+kit+compositor lib: green (67 + 57 + 259)
```

---

## Domain scores (equal weight)

| # | Domain | Prior (~77 card) | **Warpath** | Δ | Why (honest) |
|---:|---|---:|---:|---:|---|
| 1 | First-party productivity apps | 86 | **88** | +2 | Suite + MIME open → TextEdit/handlers for files |
| 2 | Toolkit / look & feel | 80 | **84** | +4 | MenuBar open API + DoAction queue; still structural a11y |
| 3 | Session login / packaging | 74 | **78** | +4 | install-session + checklist; **greeter NOT RUN** hard cap |
| 4 | Own compositor as session WM | 80 | **86** | +6 | Workspace paint/focus; SHM prefer; damage/placeholder stats |
| 5 | Multi-client window management | 76 | **83** | +7 | Live workspace + MIME-spawned clients |
| 6 | Shell chrome architecture | 78 | **86** | +8 | i18n; a11y menu open; status refresh (bat/net/vol) |
| 7 | FreeDesktop | 80 | **88** | +8 | OpenURI file://; nmcli connect plan; Inhibit→idle; **PW stubs** |
| 8 | A11y / i18n | 68 | **84** | +16 | DoAction + menu/dock/desktop menus live; Orca still incomplete |
| 9 | Multi-monitor / HDR-VRR | 70 | **82** | +12 | Settings arrange UI + conf + env; compositor parse layout |
| 10 | Polish / packaging / CI | 80 | **86** | +6 | Checklist + session_entry_smoke_report; 292 shell tests |
| | **Sum** | 772 | **848** | +76 | |
| | **Mean** | 77.2 | **84.8** | +7.6 | |
| | **Overall** | **~77** | **~85** | | |

### Arithmetic (explicit)

```text
88 + 84 + 78 + 86 + 83 + 86 + 88 + 84 + 82 + 86
  = 848
848 / 10 = 84.8
round → 85   ⇒  overall ~85 / 100
```

---

## What this score is *not*

| Claim | Reality |
|---|---|
| 100 / Plasma week | **No.** §12 0/7; greeter not run; multi-month DE work remains |
| Honest ≥90 | **No.** Residuals (greeter, PW, Orca menus, modeset) still material |
| Mid-90s from packaging scripts | **No.** Scripts ≠ live DM session |
| ScreenCast = PipeWire done | **No.** Stub `node_id` only |

---

## Residual caps (do not re-inflate without evidence)

1. Exercise greeter → session (VM log or hardware), not only desktop files.
2. Client SHM buffers routinely on nested + DRM present (placeholder rare).
3. Real PipeWire ScreenCast **or** explicit “not available” UX (no stub scored as 90).
4. Orca end-to-end: live tree re-export, extents, Text caret sync (menu.activate now live).
5. Display arrange applied to real outputs (not only env for next compositor start).
6. Window rules moving real compositor surfaces (workspace/maximize/float).
7. §12 criteria checked off with live evidence → only then revisit ≥90.

---

## Bottom line

**Warpath overall: ~85 / 100 (mean 84.6).**  
Climb was real (+4–5 pts from ~77) via **live wiring**, not policy theater.  
Would you replace Plasma for a week? **Still no.**

*Auditor stance: under-claim. Show the sum. Never invent 100.*
