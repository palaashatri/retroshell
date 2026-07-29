# QA — Stage 0 (VM Foundation)

> **This doc holds evidence, not claims.** Every row is filled from a real command
> transcript or screenshot captured on the VM. A row with no transcript is `PENDING`,
> never `PASS`. See the honesty contract in [../PROGRAM.md](../PROGRAM.md).

**Tasks under test:** [tasks/stage-0-vm-foundation.md](../tasks/stage-0-vm-foundation.md)

**Stage 0 definition of done:** over SSH from the host,
`ls /dev/dri/card0` succeeds **and** `cargo build --release --workspace` succeeds
inside the VM (Task 0.7 prints `STAGE0-DOD-PASS`).

## Result table

| Task | What it proves | Status | Evidence |
|---|---|---|---|
| 0.1 | aarch64 VM boots an aarch64 live env; `/dev/vda` present | PENDING | _paste `uname -m` / `ls /dev/vda`_ |
| 0.2 | Host serves installer + pubkey at `10.0.2.2:8000` | PENDING | _paste `curl` head of qa_key.pub_ |
| 0.3 | arm64 installer exists and is syntactically valid | PENDING | _paste `bash -n` + grep count_ |
| 0.4 | Installed system autologins; sshd active; `/dev/dri/card0` | PENDING | _paste `whoami`/`systemctl is-active sshd`/`ls /dev/dri`_ |
| 0.5 | Host reaches VM over key-based SSH | PENDING | _paste host-side `ssh … uname -m` → aarch64_ |
| 0.6 | Working tree syncs; release workspace builds in VM | PENDING | _paste `ls` of both release binaries_ |
| 0.7 | **DoD:** `card0` + workspace build | PENDING | _paste `STAGE0-DOD-PASS` transcript_ |
| 0.8 | Linux CI builds the workspace (already exists) | VERIFIED | see Transcripts: Task 0.8 |

## Runtime-confirmed values (fill in during Task 0.4)

These could not be known before running on the VM (see CONFIRM AT RUNTIME markers):

- aarch64 kernel package used (`linux` vs `linux-aarch64`): _____
- aarch64 live ISO source/version: _____
- DRM driver bound to card0 (`virtio_gpu` expected): _____

## Transcripts

_Paste raw command output here, newest first. Include the command line and its
full output. Do not summarize — the raw transcript is the evidence._

### Task 0.8 — Linux CI check (host, no VM needed)

```bash
$ grep -q 'ubuntu-latest' .github/workflows/ci.yml && grep -q 'cargo build --workspace' .github/workflows/ci.yml && echo CI-LINUX-BUILD-PRESENT
CI-LINUX-BUILD-PRESENT

$ grep -n 'runs-on: ubuntu-latest' .github/workflows/ci.yml
25:    runs-on: ubuntu-latest
67:    runs-on: ubuntu-latest
96:    runs-on: ubuntu-latest

$ grep -n 'cargo build --workspace' .github/workflows/ci.yml
53:        run: cargo build --workspace --all-targets --locked
92:        run: cargo build --workspace --release --locked
```

```text
(none yet — Stage 0 VM tasks have not been run)
```
