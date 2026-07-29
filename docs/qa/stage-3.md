# QA — Stage 3 (`.app` bundles + app store)

> **This doc holds evidence, not claims.** A row with no transcript is `PENDING`,
> never `PASS`. See the honesty contract in [../PROGRAM.md](../PROGRAM.md).

**Tasks under test:** [tasks/stage-3-app-bundles.md](../tasks/stage-3-app-bundles.md)

**Stage 3 definition of done (spec §4):** on the VM, the store installs a `.app`,
it appears in Finder/dock, and it launches — proven by a screenshot and a
transcript showing the app was installed *by the store* (not pre-placed).

**Stage status: PENDING** (authored 2026-07-30, not yet executed).

## Result table

| Task | What it proves | Status | Evidence |
|---|---|---|---|
| 3.0 | Baseline confirmed (stub + shell-out present) | PENDING | _paste `STAGE3-BASELINE-CONFIRMED`_ |
| 3.1 | `Info.toml` → `AppBundle` parser + tests | PENDING | _paste `cargo test -p retro-shell bundle::`_ |
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
(none yet — Stage 3 has not been run)
```
