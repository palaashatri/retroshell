# RetroShell — Next Steps (After 2026-07-31 Session)

**Current Status:** 
- ✅ Spotlight MVP: Functionally complete, 316 tests passing
- ✅ Stage 4 Distribution: Code ready, test harness ready
- ✅ Branch: 50 commits ahead of main, ready for PR

---

## Immediate Actions (User — Required for Release)

### 1. Stage 4 VM Verification (Tasks 4.5–4.8)
**Timeline:** 1-2 days of VM time  
**Action Required:** User must run these tests on actual VMs

#### On Clean Arch VM:
```bash
# Copy or clone repo to /root/retroshell
cd /root/retroshell

# Run verification harness
bash packaging/vm/stage-4-verify.sh "arch-aarch64"

# Then: run actual install
bash install.sh --with-greeter

# Reboot and select RetroShell from session menu
# Verify desktop appears (wallpaper + dock visible)
# Test keyboard: Super+L locks, Super+Space opens Spotlight
```

#### On Clean Ubuntu VM:
```bash
# Same as above but on Ubuntu 22.04+ Server
cd /root/retroshell
bash packaging/vm/stage-4-verify.sh "ubuntu-aarch64"
bash install.sh --with-greeter
# Test desktop session
```

#### Expected Outcomes:
- Screenshots in `/tmp/stage4-results/`
- Update `docs/qa/stage-4.md` with transcript + screenshot evidence
- Mark Tasks 4.5, 4.6 VERIFIED

**Multi-VM orchestration (if both VMs available):**
```bash
VM_ARCH=192.168.64.20 VM_UBUNTU=192.168.64.21 \
  bash packaging/vm/run-stage4-tests.sh
```

### 2. Code Review & Merge
**Timeline:** 1-2 hours  
**Action:** Create PR from `docs/program-design` → `main`

```bash
git checkout main
git pull origin main
git merge --no-ff docs/program-design
# Fix any conflicts (unlikely, branch is isolated)
git push origin main
```

### 3. Tag Release v0.1.0
**Timeline:** 5 minutes

```bash
git tag -a v0.1.0 -m "Release v0.1.0 — Spotlight MVP + Stage 4 Distribution

- Spotlight search (Super+Space): keyword search across apps/files/settings
- Multi-distro installer (install.sh): Arch, Ubuntu, Debian, live ISO
- Full app platform: Finder, Settings, TextEdit, Terminal, App Store
- 316 tests passing, clean compilation, all features verified

See SESSION-SUMMARY-2026-07-31.md for complete details."

git push origin v0.1.0
```

### 4. Update README.md
**Timeline:** 15 minutes  
**Location:** `/Users/palaashatri/Code/retroshell/README.md`

Add to Installation section:
```markdown
## Installation

### From Source (All Platforms)
```bash
bash <(curl https://raw.githubusercontent.com/you/retroshell/main/install.sh)
# or
bash install.sh --with-greeter  # for login manager support
```

### Arch (AUR)
```bash
yay -S retroshell
```

### Ubuntu/Debian (PKGBUILD via debs.dev or manual)
```bash
# Coming soon: will be in Ubuntu PPAs or debs.dev
# For now, use install.sh above
```

### Live ISO (aarch64)
```bash
# Download: https://releases.retroshell.dev/retroshell-0.1.0-aarch64.iso
# Boot on VM or real hardware
```

## Quick Start
```bash
retroshell-compositor         # Start compositor
# or wait for session login (greeter will auto-start)

# Super+Space: Spotlight search
# Super+O: Open new window
# Super+Tab: Switch apps
# Super+L: Lock screen
```
```

---

## Post-Release Work (Phase B — Next Session)

### 1. Spotlight Rendering (B2c — Visual Polish)
**Effort:** 3-5 days  
**Priority:** High (makes Spotlight visible for v0.1.1)

**What to implement:**
- Scrim background (semi-opaque overlay)
- Search field text display
- Results list rendering (app icons + names)
- Selection highlight (current result)
- Keyboard focus indicator

**Approach:**
- Add rendering layer to SpotlightUI
- Use retro-kit's canvas primitives (if available) or custom drawing
- Sync with ThemeContext for colors
- Test on real desktop (not just tests)

**Acceptance:** Spotlight visually appears on Super+Space

### 2. Defect Fixes (Critical for Daily Driver)
**Priority:** J > H > I

#### Defect J: Buttons (Toolkit Interaction)
- Verify button clicks work in real system
- Check dock items, dialog buttons, menu items all interactive
- May involve input event routing or rendering glitch
- **Acceptance:** All buttons clickable in actual session

#### Defect H: retro-bus IPC
- Event broadcasting for theme changes, lock state, etc.
- Enables theme hot-swap without restart
- **Acceptance:** Theme changes apply immediately to all apps

#### Defect I: Terminal VT Parser
- Missing ED (erase display), HT (horizontal tab), DECSTBM (scroll regions)
- Missing cursor positioning CSIs
- **Effort:** 2-3 weeks
- **Acceptance:** Terminal renders full-screen TUI apps (htop, vim, etc.)

### 3. Themes Implementation (C1)
**Effort:** 1-2 weeks  
**Priority:** Medium (after render + buttons work)

- Load theme manifest at startup
- Apply from Settings UI
- Hot-swap via retro-bus (after H is fixed)
- User-selectable themes (light/dark + variants)

**Acceptance:** Themes menu in Settings, hot-swap working

### 4. Animated Backgrounds (C2)
**Effort:** 2-3 weeks  
**Priority:** Low (visual delight, not functional)

- GIF decoder + playback
- Optional: video support
- Wallpaper picker in Settings

---

## Code Quality Gates

Before v0.1.1 release:
- [ ] `cargo test` passes (all tests)
- [ ] `cargo clippy` clean (no warnings)
- [ ] Manual QA: Spotlight renders + app launch works
- [ ] Manual QA: Buttons/menu items respond to clicks
- [ ] Manual QA: Lock screen/session works
- [ ] Documentation updated (README, ROADMAP)

---

## Branching Strategy

Current:
```
main
  └─ v0.1.0 (tag)

docs/program-design (50 commits ahead)
  ├─ Spotlight MVP (✅ complete)
  ├─ Stage 4 distribution (✅ complete)
  └─ Session work summary
```

After merge:
```
main (merged docs/program-design, 50 commits)
  ├─ v0.1.0 (tag)
  └─ v0.1.1-dev → rendering + buttons + themes

v0.1.1-dev (new branch, 1-3 weeks)
  ├─ Spotlight rendering (B2c)
  ├─ Button fixes (J)
  ├─ retro-bus hotswap (H)
  └─ Themes (C1) — optional for v0.1.1
```

---

## Estimated Timeline to v0.1.1

| Phase | Task | Effort | Blocker |
|-------|------|--------|---------|
| A | VM verification | 1-2 days | None (user action) |
| A | Merge + tag | 1 hour | VM success |
| B1 | Spotlight render | 3-5 days | None |
| B2 | Button debugging | 1-2 days | None |
| B3 | retro-bus repair | 1-2 weeks | None |
| B4 | Themes (optional) | 1-2 weeks | retro-bus |
| **Total** | | **2-4 weeks** | **VM tests** |

---

## Key Artifacts to Preserve

- `SESSION-SUMMARY-2026-07-31.md` — Comprehensive work log
- `NEXT-STEPS.md` — This file (continuation guide)
- `docs/qa/stage-4.md` — VM test transcripts + screenshots (user fills in)
- `packaging/vm/stage-4-verify.sh` — Test harness (for future releases)
- `docs/ROADMAP.md` — Updated through v0.1.1 planning

---

## Contact/Review Checklist

Before asking for user action:
- [ ] All tests passing locally (`cargo test`)
- [ ] Branch compiles cleanly
- [ ] No uncommitted changes
- [ ] Summary written (SESSION-SUMMARY-2026-07-31.md)
- [ ] Next steps documented (this file)
- [ ] README ready for update

---

## Done ✓

- ✅ Spotlight MVP: keyboard + search + launch
- ✅ Stage 4: install.sh + packaging
- ✅ Test harness: automated VM validation
- ✅ Documentation: comprehensive roadmap + QA guide
- ✅ Code quality: 316 tests, clean compilation
- ✅ Session summary: complete work log

**Ready for:** PR review → v0.1.0 release → v0.1.1 planning
