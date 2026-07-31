# QA — Stage 0 (VM Foundation)

> **This doc holds evidence, not claims.** Every row is filled from a real command
> transcript or screenshot captured on the VM. A row with no transcript is `PENDING`,
> never `PASS`. See the honesty contract in [../SLOPOS-I.md](../SLOPOS-I.md).

**Tasks under test:** [tasks/stage-0-vm-foundation.md](../tasks/stage-0-vm-foundation.md)
(Mac/UTM Path A/B in that doc are **dormant**; this run used the Windows +
VirtualBox pipeline from [SLOPOS-I.md](../SLOPOS-I.md) §3.)

**Stage 0 definition of done:** over SSH from the host,
`ls /dev/dri/card0` succeeds **and** `cargo build --release --workspace` succeeds
inside the VM (prints `STAGE0-DOD-PASS`).

**Stage status: VERIFIED** (2026-07-30, Windows host + VirtualBox guest).

## Result table (Windows + VirtualBox / x86_64)

| Check | What it proves | Status | Evidence |
|---|---|---|---|
| VM exists | `slopos-i-arch` with EFI, VMSVGA+3D, NAT `:2222→22` | VERIFIED | VBoxManage showvminfo (below) |
| Autologin | boots to `retro` on tty1 | VERIFIED | screenshot + `whoami` |
| SSH key | host reaches guest with `qa_key`, no password | VERIFIED | `uname -m` → `x86_64` |
| KMS | `/dev/dri/card0` + `vmwgfx` | VERIFIED | transcript below |
| Workspace build | `cargo build --release --workspace` | VERIFIED | `STAGE0-DOD-PASS` |
| 0.8 CI | Linux CI builds workspace | VERIFIED | see Transcripts: Task 0.8 |

Mac/UTM Path A/B tasks in the Stage-0 task doc remain **UNVERIFIED** (dormant on
this machine; not required for Stage 0 DoD here).

## Runtime-confirmed values (Windows path)

- Host: Windows x86_64, VirtualBox 7.2.12
- Guest arch: `x86_64`
- DRM driver bound to card0: **`vmwgfx`**
- Disk: `/dev/sda2` (~59G root)
- User: `retro` (autologin tty1); SSH via `packaging/vm/qa_key`

## Transcripts

_Paste raw command output here, newest first. Include the command line and its
full output. Do not summarize — the raw transcript is the evidence._

### Stage 0 DoD (2026-07-30, Windows host)

```text
$ ssh -i packaging/vm/qa_key -p 2222 retro@127.0.0.1 \
    'ls /dev/dri/card0 && lsmod | grep vmwgfx | head -1 && cd ~/slopos-i && cargo build --release --workspace && echo STAGE0-DOD-PASS'
/dev/dri/card0
vmwgfx                491520  0
… (crate downloads + compile) …
    Finished `release` profile [optimized] target(s) in 5m 32s
STAGE0-DOD-PASS
```

Follow-up probe:

```text
$ ssh -i packaging/vm/qa_key -p 2222 retro@127.0.0.1 \
    'ls /dev/dri/card0 && lsmod | grep vmwgfx; ls -1 ~/slopos-i/target/release/slopos-compositor ~/slopos-i/target/release/slopos-shell; uname -m; whoami; systemctl is-active sshd'
/dev/dri/card0
vmwgfx                491520  0
drm_ttm_helper         20480  2 vmwgfx
ttm                   151552  2 vmwgfx,drm_ttm_helper
/home/retro/slopos-i/target/release/slopos-compositor
/home/retro/slopos-i/target/release/slopos-shell
x86_64
retro
active
```

### VM hardware (host)

```text
name="slopos-i-arch"
memory=8192
firmware="EFI"
cpus=4
graphicscontroller="vmsvga"
accelerate3d="on"
NIC 1 Rule(0): ssh … host port = 2222 … guest port = 22
Boot Device 1: HardDisk
```

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
