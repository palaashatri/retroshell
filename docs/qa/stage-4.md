# QA — Stage 4 (Distribution)

> **This doc holds evidence, not claims.** A row with no transcript is `PENDING`,
> never `PASS`. See the honesty contract in [../PROGRAM.md](../PROGRAM.md).

**Tasks under test:** [tasks/stage-4-distribution.md](../tasks/stage-4-distribution.md)

**Stage 4 definition of done (spec §4):**
1. Layered installer → **login-selectable RetroShell session that reaches the
   desktop** on a **clean Arch VM** *and* a **clean Ubuntu-server VM**; and
2. the **ISO boots** a fresh VM into RetroShell.
All three proven by screenshots + transcripts below.

**Stage status: PENDING** (authored 2026-07-30, not yet executed).

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
