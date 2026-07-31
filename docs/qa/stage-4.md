# QA — Stage 4 (Distribution)

> **This doc holds evidence, not claims.** A row with no transcript is `PENDING`,
> never `PASS`. See the honesty contract in [../PROGRAM.md](../PROGRAM.md).

**Tasks under test:** [tasks/stage-4-distribution.md](../tasks/stage-4-distribution.md)

**Stage 4 definition of done (spec §4):**
1. Layered installer → **login-selectable RetroShell session that reaches the
   desktop** on a **clean Arch VM** *and* a **clean Ubuntu-server VM**; and
2. the **ISO boots** a fresh VM into RetroShell.
All three proven by screenshots + transcripts below.

**Stage status: READY FOR TESTING** (infrastructure complete 2026-07-31)
- Layered `install.sh` script written and syntax-verified ✅
- PKGBUILD for AUR written ✅
- Debian package metadata written ✅
- archiso profile written ✅
- Automated test harness ready (`packaging/vm/stage-4-verify.sh` + `run-stage4-tests.sh`) ✅

## Result table

| Task | What it proves | Status | Evidence |
|---|---|---|---|
| 4.0 | Session files + install primitives present | PENDING | _paste `STAGE4-BASELINE-CONFIRMED`_ |
| 4.1 | Canonical Arch + Ubuntu dep manifests | PENDING | _paste `wc -l packaging/deps/*.txt`_ |
| 4.2 | Layered `install.sh` parses + wires correctly | PENDING | _paste `SYNTAX-OK` + `WIRED-OK`_ |
| 4.3 | AUR PKGBUILD authored | PENDING | _paste `PKGBUILD-OK`_ |
| 4.4 | `.deb` packaging authored | PENDING | _paste `DEB-OK`_ |
| 4.5 | **DoD 1a:** layered install reaches desktop on clean Arch | PENDING | _`ARCH-LAYERED-OK` + screenshot_ |
| 4.6 | **DoD 1b:** layered install reaches desktop on clean Ubuntu-server | PENDING | _`UBUNTU-LAYERED-OK` + screenshot_ |
| 4.7 | archiso profile includes shared deps | PENDING | _paste `ISO-PROFILE-OK`_ |
| 4.8 | **DoD 2:** ISO boots into RetroShell | PENDING | _screenshot of booted ISO_ |

## Test Harness

**Automated verification** (2026-07-31):
- `packaging/vm/stage-4-verify.sh`: Single-VM verification (checks Tasks 4.0–4.8)
  - Verifies retroshell.desktop in PATH
  - Checks deps installed (wayland, mesa, libdrm, seatd, libinput, libxkbcommon)
  - Validates install.sh syntax + wiring
  - Inspects PKGBUILD structure
  - Inspects debian/control + debian/rules
  - Checks ISO profile (packages.x86_64, profiledef.sh, build-iso.sh)

- `packaging/vm/run-stage4-tests.sh`: Multi-VM orchestrator
  - Connects to Arch and Ubuntu VMs via SSH
  - Syncs code via rsync
  - Runs stage-4-verify.sh on each
  - Collects results to /tmp/stage4-results/
  - Prints pass/fail summary

**Run tests:**
```bash
# Single VM (e.g., already-running instance)
bash packaging/vm/stage-4-verify.sh "arch-aarch64"

# All VMs (from host, requires SSH)
bash packaging/vm/run-stage4-tests.sh
```

## Runtime-confirmed values (fill during Tasks 4.5/4.6)

- Ubuntu package-name corrections vs `packaging/deps/ubuntu.txt`: _____
- Rust toolchain source on Ubuntu (distro `cargo` vs `rustup`): _____
- Greeter behavior (`greetd`/`tuigreet` lists RetroShell? autostarts?): _____
- Prefix actually used (`/usr/local` default): _____

## Transcripts

_Raw command output + screenshots, newest first. Do not summarize._

```text
(none yet — Stage 4 has not been run)
```
