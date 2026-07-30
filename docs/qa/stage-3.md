# QA — Stage 3 (`.app` bundles + app store)

> **This doc holds evidence, not claims.** A row with no transcript is `PENDING`,
> never `PASS`. See the honesty contract in [../PROGRAM.md](../PROGRAM.md).

**Tasks under test:** [tasks/stage-3-app-bundles.md](../tasks/stage-3-app-bundles.md)

**Stage 3 definition of done (spec §4):** on the VM, the store installs a `.app`,
it appears in Finder/dock, and it launches — proven by a screenshot and a
transcript showing the app was installed *by the store* (not pre-placed).

**Stage status: IN PROGRESS** (Task 3.0 baseline confirmed 2026-07-30; Task 3.1 implemented).

## Result table

| Task | What it proves | Status | Evidence |
|---|---|---|---|
| 3.0 | Baseline confirmed (stub + shell-out present) | PASS | `STAGE3-BASELINE-CONFIRMED` (branch `docs/program-design`) |
| 3.1 | `Info.toml` → `AppBundle` parser + tests | PASS | _paste `cargo test -p retro-shell bundle::` transcript below_ |
| 3.2 | `scan_applications` reads `*.app` from disk | PENDING | _paste retro-shell test result_ |
| 3.3 | Launch execs `<path>/<entrypoint>` | PENDING | _paste build + test result_ |
| 3.4 | One `.app` assembled by script | PENDING | _paste `BUNDLE-BUILD-OK`_ |
| 3.5 | All 5 first-party apps packaged | PENDING | _paste `5`_ |
| 3.6 | Package-manager path removed (spec §5.3) | PENDING | _paste `PACKAGE-PATH-REMOVED`_ |
| 3.7 | `.app` installer (sha256/extract/atomic) + tests | PENDING | _paste `cargo test -p appstore bundle_install::`_ |
| 3.8 | Install button wired to `.app` installer | PENDING | _paste `INSTALL-WIRED`_ |
| 3.9 | (optional) HTTP fetch builds | PENDING | _paste build `Finished`_ |
| 3.10 | **DoD:** store installs a `.app`; it shows + launches on VM | PENDING | _`INSTALLED-VIA-STORE` + screenshot_ |

## Runtime-confirmed values (fill during Task 3.10)

- How the running shell triggers a `scan_applications` rescan after install: _____
- Install target actually used (`~/Applications` expected): _____
- Catalog source used (local path vs `file://` vs http): _____

## Transcripts

_Raw command output, newest first. Do not summarize — the transcript is the
evidence._

```text
# Task 3.0 — baseline (host, 2026-07-30)
$ git rev-parse --abbrev-ref HEAD
docs/program-design
$ grep -q 'For now, register built-in apps' crates/retro-shell/src/launch_services.rs && \
  grep -q 'RETROSHELL_APPSTORE_ALLOW_PACKAGE_CHANGES' apps/appstore/src/main.rs && \
  echo STAGE3-BASELINE-CONFIRMED
STAGE3-BASELINE-CONFIRMED

# Task 3.1 — bundle parser (VM, 2026-07-30)
$ cargo test -p retro-shell bundle:: 2>&1 | grep -E 'test result:'
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 300 filtered out; finished in 0.01s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s
```
