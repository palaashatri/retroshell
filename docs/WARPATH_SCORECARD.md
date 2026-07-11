# Warpath scorecard (honest arithmetic)

**Date:** 2026-07-11  
**Question:** After the live-integration warpath (post ~90 rejection), what is the
honest equal-weight daily-driver score vs Plasma/GNOME?

**Answer: ~82 / 100 (mean 81.6).** Not 90. Not 100. Prefer under-claim.

Related: [`DEEP_AUDIT_90_CLAIM.md`](DEEP_AUDIT_90_CLAIM.md),
[`implementation_plan.md`](implementation_plan.md) §13, [`README.md`](../README.md).

---

## Methodology

- **100** = replace Plasma for a week without constant workarounds.
- **10 domains, equal weight.** Overall = arithmetic mean; “~NN” = mean rounded
  to nearest integer.
- **Pure module + unit test without a live path does not score as domain 90.**
- Live greeter **NOT RUN**, PipeWire streams **stubs**, `chrome.menu.activate`
  **log-only**, §12 **0 / 7** fully met — these **cap** several domains.

---

## Timeline (same methodology)

| Milestone | Mean | Overall |
|---|---:|---:|
| Inflated “hard DE ~90” card | 89.6 | **~90 WITHDRAWN** (score theater) |
| Deep audit (claim rejection) | 75.6 | **~76** |
| Immediate wire (filter + portal + lock i18n) | 77.2 | **~77** |
| **Warpath rescore (this card)** | **81.6** | **~82** |

---

## Warpath evidence (verified in-tree)

| Landing | Status | Where |
|---|---|---|
| Workspace paint / focus in compositor main | **live** | `crates/retro-compositor/src/main.rs` (`windows_visible_for_paint`, hit-test); `workspace_focus.rs` |
| Per-window SHM vs placeholder | **prefer SHM** | `window_paint_source`; placeholder only when surface tree empty |
| `RETROSHELL_OUTPUTS_LAYOUT` shell → compositor | **env bridge** | shell `apply_display_plan_env`; compositor `parse` / layout source |
| DoAction queue → shell handlers | **partial live** | `drain_a11y_pending_actions` → lock / log_out / force_quit / workspace / window close·activate / dock / desktop |
| `chrome.menu.activate` | **log-only stub** | `a11y_invoke_is_live` false; `dispatch_a11y_invoke` debug only |
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
| 1 | First-party productivity apps | 86 | **86** | 0 | Real suite + Force Quit; not the DE bottleneck |
| 2 | Toolkit / look & feel | 80 | **83** | +3 | DoAction pending queue kit→shell; a11y still structural |
| 3 | Session login / packaging | 74 | **78** | +4 | install-session + checklist; **greeter NOT RUN** hard cap |
| 4 | Own compositor as session WM | 80 | **84** | +4 | Workspace filter paint/focus live; SHM prefer; placeholders remain |
| 5 | Multi-client window management | 76 | **81** | +5 | Live workspace hide + focus; rules partial on real surfaces |
| 6 | Shell chrome architecture | 78 | **83** | +5 | i18n system menus; chrome a11y dispatch; layer dual path remains |
| 7 | FreeDesktop | 80 | **84** | +4 | Inhibit→idle wired; Secret/Print plan-level; **PW stubs** |
| 8 | A11y / i18n | 68 | **77** | +9 | Menus+lock `tr()`; DoAction live set; menu.activate stub; Orca incomplete |
| 9 | Multi-monitor / HDR-VRR | 70 | **76** | +6 | LAYOUT shell→comp; no live modeset / Settings apply UI |
| 10 | Polish / packaging / CI | 80 | **84** | +4 | Tests + Docker + install-session + daily checklist |
| | **Sum** | 772 | **816** | +44 | |
| | **Mean** | 77.2 | **81.6** | +4.4 | |
| | **Overall** | **~77** | **~82** | | Round half up to nearest int |

### Arithmetic (explicit)

```text
86 + 83 + 78 + 84 + 81 + 83 + 84 + 77 + 76 + 84
  = 816
816 / 10 = 81.6
round → 82   ⇒  overall ~82 / 100
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
4. `chrome.menu.activate` live open; Orca drives core chrome end-to-end.
5. Display arrange applied to real outputs (not only env for next compositor start).
6. Window rules moving real compositor surfaces (workspace/maximize/float).
7. §12 criteria checked off with live evidence → only then revisit ≥90.

---

## Bottom line

**Warpath overall: ~82 / 100 (mean 81.6).**  
Climb was real (+4–5 pts from ~77) via **live wiring**, not policy theater.  
Would you replace Plasma for a week? **Still no.**

*Auditor stance: under-claim. Show the sum. Never invent 100.*
