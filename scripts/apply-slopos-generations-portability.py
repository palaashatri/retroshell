#!/usr/bin/env python3
"""Insert the accepted SLOPOS-I/SLOPOS-II portability contract into AGENTS.md.

This is a one-shot guarded migration. It refuses to run when anchors differ,
so it cannot silently duplicate or corrupt the sole normative development file.
"""

from pathlib import Path

path = Path(__file__).resolve().parents[1] / "AGENTS.md"
text = path.read_text(encoding="utf-8")


def replace_once(old: str, new: str, label: str) -> None:
    global text
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected one occurrence, found {count}")
    text = text.replace(old, new, 1)


replace_once(
    "SLOPOS-I is a sovereign, local-first Linux desktop environment written in Rust\n"
    "and, where justified, assembly. It combines the visual and interaction lineage\n"
    "of classic Macintosh System 7 / Platinum with the architecture and expected\n"
    "capabilities of a modern KDE/GNOME-class desktop.\n",
    "SLOPOS-I is a sovereign, local-first, Linux-first desktop environment written in\n"
    "Rust and, where justified, assembly. Its shared userland and desktop policy must\n"
    "remain POSIX-portable so the same desktop can run on Linux and FreeBSD without a\n"
    "fork. It combines the visual and interaction lineage of classic Macintosh System\n"
    "7 / Platinum with the architecture and expected capabilities of a modern\n"
    "KDE/GNOME-class desktop.\n",
    "project identity",
)

lineage = r'''
### Product generations and release milestones

SLOPOS is one desktop product lineage with kernel support added in generations.
A generation is not permission to rewrite the desktop, abandon compatibility,
or reset already completed functionality.

#### SLOPOS-I — desktop-environment generation

The first release milestone is a complete, sovereign **Linux desktop
environment**. Linux is the Tier-1 reference platform and the compositor's
first 100/100 implementation target.

SLOPOS-I must also establish a real POSIX/Unix platform boundary and bring the
same desktop to FreeBSD. The order is:

1. **SLOPOS-I M1 — Linux desktop:** complete compositor, shell, toolkit,
   applications, session, packaging, accessibility and daily-driver QA on
   Linux. No third-party production compositor.
2. **SLOPOS-I M2 — portable desktop:** shared crates are POSIX-clean, required
   release scripts are POSIX `sh`, Linux-specific services are isolated behind
   platform interfaces, and a native FreeBSD backend builds and runs the same
   desktop experience.

Linux and FreeBSD are operating-system substrates for SLOPOS-I. SLOPOS-I does
not include a custom kernel.

#### SLOPOS-II — custom-kernel generation

The **only generational objective** of SLOPOS-II is to add a first-party custom
Rust kernel as a third supported kernel target. SLOPOS-II is not a UI redesign,
application rewrite, compatibility break, or excuse to regress Linux or FreeBSD.

The SLOPOS-II support matrix is mandatory:

| Kernel target | Required status |
|---|---|
| Linux | Remains fully supported and release-blocking |
| FreeBSD | Remains fully supported and release-blocking |
| SLOPOS kernel | New first-party Rust/assembly kernel and release-blocking target |

The desktop, shell, compositor policy, toolkit, SDK, applications, document
formats, accessibility semantics and user configuration must remain shared.
Kernel-specific code belongs behind platform and ABI adapters. Do not create
three application trees or three competing desktop implementations.

A kernel alone cannot honestly be called POSIX-compliant. POSIX conformance is
a system property involving kernel behavior, libc/API surfaces, the shell and
utilities. Therefore the SLOPOS-II program includes only the minimum companion
work required to expose and verify a POSIX-conformant system interface for the
custom kernel: processes and threads, virtual memory, filesystems and VFS,
permissions, signals, clocks/timers, pipes, Unix sockets, networking, device and
terminal interfaces, executable loading, system calls, libc bindings, and the
required command/runtime surface.

Use **POSIX-conformant target** until the relevant conformance suites pass. Use
**POSIX certified** only after formal certification has actually been obtained.
No documentation may infer certification from design intent or unit tests.

SLOPOS-II may begin only after SLOPOS-I has a stable Linux desktop, a frozen
portable platform contract, and a non-regression suite capable of running the
same desktop/application tests on Linux and FreeBSD. The custom kernel must then
join that same matrix; it must not replace either existing kernel target.

'''

replace_once(
    "### Naming\n",
    lineage + "### Naming\n",
    "generation insertion",
)

portability = r'''
### POSIX and operating-system portability contract

POSIX does not specify Wayland, DRM/KMS, desktop composition, window controls,
SLOPOS Spaces, graphical applications or visual design. Do not describe those
GUI features as POSIX features. The enforceable goal is a POSIX-portable shared
userland with explicit operating-system backends.

#### Required architecture

```text
Shared SLOPOS desktop and POSIX/Unix layer
├── compositor policy and Wayland protocol state
├── shell and applications
├── toolkit, SDK, renderer-independent scene policy
├── file/process/IPC abstractions
├── configuration, bundles and document services
├── Vision protocol/client and portable inference core
└── platform traits
    ├── Linux backend
    ├── FreeBSD backend
    └── SLOPOS-kernel backend (SLOPOS-II only)
```

Create or evolve explicit boundaries equivalent to:

```text
crates/slopos-platform
crates/slopos-platform-linux
crates/slopos-platform-freebsd
```

The future SLOPOS-II repository/program adds a SLOPOS-kernel implementation of
the same public platform contract. Names may change during implementation, but
the dependency direction may not: shared desktop crates depend on interfaces,
not Linux, FreeBSD or SLOPOS-kernel implementations.

Portable crates must not directly depend on `/proc`, `/sys`, udev, systemd,
logind, epoll, inotify, signalfd, memfd-specific behavior, Linux credential
structures, Linux DRM ioctls, NetworkManager, PipeWire, or Linux-only command
output. Those facilities are allowed only inside the Linux backend. FreeBSD and
future SLOPOS-kernel facilities receive their own implementations.

General Unix APIs may use `std::os::unix` and carefully reviewed `libc` calls.
Linux-only APIs must be under `cfg(target_os = "linux")` in Linux-owned modules.
FreeBSD-only APIs must be isolated likewise. A broad `cfg(unix)` is not proof
that behavior is portable.

#### Shell and command portability

Every script required to build, install, start, stop, recover, upgrade, package
or test a supported release must use POSIX shell syntax unless it is explicitly
platform-owned:

```sh
#!/bin/sh
set -eu
```

Do not require Bash arrays, `[[ ... ]]`, `${BASH_SOURCE[0]}`, process
substitution, `set -o pipefail`, GNU-only `stat`, GNU-only `sed`, `grep -P`,
`readlink -f`, `timeout`, or `seq` in the portable release path. A Linux-only
developer/QA script may use Bash, but it must be labelled as such and may not be
the sole route to build or operate SLOPOS-I on FreeBSD.

#### Portability gates

CI must grow to include:

- Linux glibc workspace build/test;
- Linux musl portability build where dependencies permit;
- FreeBSD workspace build/test on a native runner or VM;
- POSIX-shell validation under at least `dash` and BusyBox `ash` for portable
  scripts, plus FreeBSD `/bin/sh` when the runner exists;
- a dependency-boundary check that rejects Linux-only imports from portable
  crates;
- shared behavioral tests for filesystem, process, IPC, settings and session
  abstractions;
- identical first-party application tests across Linux and FreeBSD;
- in SLOPOS-II, the same non-regression suite against the SLOPOS kernel.

Do not claim FreeBSD support from `cargo check` alone. Full support requires a
native compositor/session, input, graphics, audio, power, networking, packaging
and application runtime evidence. Do not claim SLOPOS-II kernel support until a
real desktop session and the shared compatibility suite run on that kernel.

'''

replace_once(
    "---\n\n## 3. Current repository map\n",
    portability + "---\n\n## 3. Current repository map\n",
    "portability architecture insertion",
)

p05 = r'''
### P0.5 — Freeze the portable platform boundary

This work starts during SLOPOS-I rather than being deferred to SLOPOS-II:

- inventory every Linux-specific import, path, command, service and protocol;
- classify it as shared Unix/POSIX behavior or platform implementation;
- define typed platform interfaces for session/seat, device discovery, display,
  input, audio, power, networking, notifications, credentials and filesystem
  integration;
- move Linux implementations behind the interface without weakening the Linux
  compositor or replacing direct hardware support with stubs;
- add FreeBSD compile gates, then native runtime implementations and evidence;
- keep application and shell code free of direct Linux service invocation;
- convert release-critical scripts to POSIX `sh` or provide an equivalent
  FreeBSD-native path;
- record all remaining platform leakage in `TRUTH.md`.

Exit gate: the Linux desktop remains fully functional, portable crates contain
no accidental Linux dependencies, and the FreeBSD backend can be implemented
without changing public application or desktop policy APIs.

'''

replace_once(
    "### P1 — Compositor and session correctness\n",
    p05 + "### P1 — Compositor and session correctness\n",
    "P0.5 insertion",
)

path.write_text(text, encoding="utf-8")
print("Applied SLOPOS generation and POSIX portability contract to AGENTS.md")
