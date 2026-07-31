# RetroShell Docs — Single Source of Truth

**Start here.** Everything else is either stage evidence, tasks, or archive.

| Doc | Role |
|-----|------|
| **[PROGRAM.md](PROGRAM.md)** | Honesty contract, stage status, architecture map |
| **[UI.md](UI.md)** | Classic Mac / System 7 visual SoT + references + current polish status |
| **[HANDOFF.md](HANDOFF.md)** | How to run the UTM/VBox VMs, build, screenshot |
| **[FUTURE.md](FUTURE.md)** | Backlog that is **not** current stage work |
| **[tasks/](tasks/)** | Atomic stage tasks (acceptance commands) |
| **[qa/](qa/)** | Stage / feature QA evidence (screenshots + transcripts) |
| **[specs/](specs/)** | Long-form design program spec |
| **[archive/](archive/)** | Superseded session notes, old roadmaps, claimed-complete theme docs |

## Rules

1. **Do not add new root-level session summaries.** Update `PROGRAM.md`, `UI.md`, or a `qa/` evidence file.
2. **A feature is done only with VM evidence** in `qa/` (honesty contract in `PROGRAM.md`).
3. **UI claims** live only in `UI.md` + `qa/ui-polish/` screenshots — nowhere else.

## Current focus (2026-07-31)

- **UI quality:** System 7–faithful paint (see `UI.md`) — still far from kit parity; keep iterating with screenshots.
- **Program stages 0–3:** verified on VM (see `PROGRAM.md`).
- **Stage 4:** code-complete; VM install/ISO verification pending (`qa/stage-4.md`).
- **Do not regress** HDR/VRR / compositor work while polishing pixels.
