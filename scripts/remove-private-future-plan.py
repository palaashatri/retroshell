#!/usr/bin/env python3
"""Remove private future-kernel planning from the public SLOPOS documents.

This is a guarded one-shot migration. It edits only AGENTS.md and TRUTH.md and
refuses to write if the expected anchors are absent or private-plan terms remain.
"""

from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]


def replace_once(text: str, pattern: str, replacement: str, label: str, *, flags=0) -> str:
    updated, count = re.subn(pattern, replacement, text, count=1, flags=flags)
    if count != 1:
        raise RuntimeError(f"{label}: expected one replacement, found {count}")
    return updated


agents_path = ROOT / "AGENTS.md"
agents = agents_path.read_text(encoding="utf-8")

agents = replace_once(
    agents,
    r"### Product generations and release milestones\n.*?(?=### Naming\n)",
    """### Release milestones

SLOPOS-I is the desktop-environment product. Its first release milestone is a
complete, sovereign Linux desktop environment. Linux is the Tier-1 reference
platform and the compositor's first 100/100 implementation target.

The second portability milestone is the same desktop on FreeBSD through a
POSIX/Unix platform boundary, without forking the shell, applications, toolkit,
SDK, compositor policy, accessibility semantics, configuration or document
formats.

1. **M1 — Linux desktop:** complete compositor, shell, toolkit, applications,
   session, packaging, accessibility and daily-driver QA on Linux. No
   third-party production compositor.
2. **M2 — portable desktop:** shared crates are POSIX-clean, required release
   scripts are POSIX `sh`, Linux-specific services are isolated behind platform
   interfaces, and a native FreeBSD backend builds and runs the same desktop
   experience.

Linux and FreeBSD are operating-system substrates for SLOPOS-I. This repository
is scoped to the desktop environment and its supporting userland.

""",
    "release milestones",
    flags=re.S,
)

agents = agents.replace("    └── SLOPOS-kernel backend (SLOPOS-II only)\n", "")
agents = replace_once(
    agents,
    r"The future SLOPOS-II repository/program adds a SLOPOS-kernel implementation of\n"
    r"the same public platform contract\. Names may change during implementation, but\n"
    r"the dependency direction may not: shared desktop crates depend on interfaces,\n"
    r"not Linux, FreeBSD or SLOPOS-kernel implementations\.\n",
    "Names may change during implementation, but the dependency direction may not: "
    "shared desktop crates depend on interfaces, not Linux or FreeBSD implementations.\n",
    "future backend paragraph",
)
agents = agents.replace(
    "Those facilities are allowed only inside the Linux backend. FreeBSD and\n"
    "future SLOPOS-kernel facilities receive their own implementations.\n",
    "Those facilities are allowed only inside the Linux backend. FreeBSD receives\n"
    "its own implementation.\n",
)
agents = agents.replace(
    "- in SLOPOS-II, the same non-regression suite against the SLOPOS kernel.\n",
    "",
)
agents = agents.replace(
    "Do not claim SLOPOS-II kernel support until a\n"
    "real desktop session and the shared compatibility suite run on that kernel.\n",
    "",
)

for forbidden in ("SLOPOS-II", "SLOPOS-kernel", "SLOPOS kernel", "custom kernel"):
    if forbidden in agents:
        raise RuntimeError(f"AGENTS.md still contains private-plan term: {forbidden}")
agents_path.write_text(agents, encoding="utf-8")

truth_path = ROOT / "TRUTH.md"
truth = truth_path.read_text(encoding="utf-8")
truth = truth.replace(
    "**SLOPOS-II implementation status:** planned only; no kernel source exists.\n",
    "",
)
truth = truth.replace(
    "| **SLOPOS-II custom kernel** | **0** | Intentionally not started. |\n",
    "",
)
truth = truth.replace(
    "## 12. POSIX, FreeBSD and SLOPOS-II truth",
    "## 12. POSIX and FreeBSD truth",
)
truth = replace_once(
    truth,
    r"\n### SLOPOS-II\n.*?(?=\n---\n\n## 13\.)",
    "",
    "private future section",
    flags=re.S,
)
truth = truth.replace(
    "8. **Start SLOPOS-II as a separate kernel programme using the frozen platform\n"
    "   contract.** Linux, FreeBSD and the SLOPOS kernel all remain release-blocking.\n",
    "",
)
truth = replace_once(
    truth,
    r"\nSLOPOS-II is correctly scoped but remains \*\*0/100 implemented\*\*\. Its future task\n"
    r"is to add a POSIX-conformant first-party Rust kernel as a third target while the\n"
    r"same SLOPOS desktop continues to pass release gates on Linux and FreeBSD\.\n",
    "\n",
    "bottom-line private plan",
)

for forbidden in ("SLOPOS-II", "SLOPOS-kernel", "SLOPOS kernel", "custom kernel"):
    if forbidden in truth:
        raise RuntimeError(f"TRUTH.md still contains private-plan term: {forbidden}")
truth_path.write_text(truth, encoding="utf-8")

print("Removed private future plans from AGENTS.md and TRUTH.md")
