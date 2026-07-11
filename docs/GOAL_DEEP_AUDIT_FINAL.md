# Goal deep audit (final refresh) — 2026-07-11

**Methodology:** equal-weight mean of **10 domains** vs Plasma/GNOME as a
**daily-driver laptop DE** (100 = replace Plasma for a week).  
**Pure helpers + unit tests without a live path do not score as domain 90.**

**Prior ~90 (mean 89.6) claim: WITHDRAWN** (score theater).

**Current overall: ~85 / 100 (mean 84.5; sum 845).** Not 90. Not 100.

**Canonical vector:** `88, 84, 78, 86, 83, 86, 88, 84, 82, 86`

Probe capture: implementer scratch `pure_vs_live_probes.txt` (same day).

---

## Pure vs live (gating evidence)

| Capability | Status | Evidence |
|---|---|---|
| Workspace composition filter | **LIVE** | `main.rs` `windows_visible_for_paint` / hit-test / Super+ws |
| Per-window SHM vs placeholder | **LIVE prefer SHM** | `window_paint_source`; placeholder only if empty surface tree |
| `RETROSHELL_OUTPUTS_LAYOUT` | **LIVE env bridge** | shell `apply_display_plan_env` + compositor `parse_outputs_layout_spec` |
| Settings display arrange | **LIVE on save** | Settings Display UI → conf + env |
| i18n `tr()` | **LIVE (partial)** | system menu + lock screen callers |
| Portal Secret/Print/Inhibit | **ON BUS (plan-level)** | `portal_dbus` serve_at; Secret/Print not keyring/CUPS |
| Inhibit → idle | **LIVE in-process** | `handle_inhibit_and_register` + shell merge |
| ScreenCast | **STUB + honesty notes** | placeholder `node_id`; `backend=portal_stub` / socket note |
| DoAction → shell | **LIVE (partial)** | queue + `dispatch_a11y_invoke` for lock/logout/FQ/ws/window/dock/desktop/**menu open** |
| `chrome.dock.menu` / `chrome.desktop.menu` | **LIVE** | status windows listing dock/desktop items |
| MIME open / OpenURI file:// | **LIVE spawn path** | `spawn_open_plan` |
| nmcli connect plan | **LIVE spawn path** | `execute_nm_connect_plan` |
| Greeter → session | **NOT RUN** | packaging + `install-session-files` only |
| §12 “we made it” | **0 / 7 fully met** | install artifacts only on criterion 1 |

---

## Scorecard (equal weight)

| Domain | Score | Why |
|---|---:|---|
| First-party productivity apps | **88** | Suite + MIME open + Force Quit |
| Toolkit / look & feel | **84** | MenuBar open API + DoAction queue |
| Session login / packaging | **78** | install-session + checklist; greeter **NOT RUN** |
| Own compositor as session WM | **86** | Workspace paint/focus; SHM prefer; damage stats |
| Multi-client window management | **83** | Live ws hide + MIME-spawned clients |
| Shell chrome architecture | **86** | i18n menus; a11y menu open; status refresh |
| FreeDesktop | **88** | OpenURI file://; nmcli; Inhibit→idle; **PW stubs** |
| A11y / i18n | **84** | DoAction live set + menu/dock/desktop menus; Orca incomplete |
| Multi-monitor / HDR-VRR | **82** | Settings arrange + compositor layout parse |
| Polish / packaging / CI | **86** | Checklist + session_entry_smoke_report; 292 shell tests |
| **Sum** | **845** | |
| **Mean** | **84.5** | |
| **Overall** | **~85** | round half up |

```text
88+84+78+86+83+86+88+84+82+86 = 845
845 / 10 = 84.5 → ~85
```

---

## Why not 90 or 100

1. Live greeter login **NOT RUN** (hard cap on session domain and §12).  
2. PipeWire ScreenCast **stubs**.  
3. Orca not end-to-end (extents/caret/live re-export).  
4. Client placeholder path still exists.  
5. §12 **0/7 fully met**.  
6. Claiming 100 under this methodology without Plasma-week workability is **score theater**.

**Contract:** If overall &lt; 90, publish honest score + residual caps; climb with live evidence only. Do not invent 100 in this environment.

---

## Multi-agent warpath commits (sample)

`00c53f9` compositor workspace SHM · `6455ec3` shell a11y/i18n/display · `622d37e` fdo inhibit ·  
`0aca984` OUTPUTS_LAYOUT · `6d177b3` packaging · `fcf11ba` a11y dispatch · `ebe63cb` menu open ·  
`6a0583a` MIME open · `c6fa5b5` damage · `d9ee0b2` settings arrange · `62fe111` nmcli ·  
`e8d9a89` docs ~85
