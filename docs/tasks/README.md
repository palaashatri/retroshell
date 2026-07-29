# Task docs — format & rules

These docs are written for an executor with **zero project context** — a small
model (Gemma-3n class) or a junior engineer. Read this once, then execute tasks
in order.

## How to execute a task

1. Read the whole task before typing anything.
2. Check the **Precondition** — run its command. If it fails, STOP and do the
   task it names first.
3. Do the **Steps** in order. Change only the files the task names.
4. Run the **Acceptance** command. Compare its output to the expected output
   **exactly**. If it does not match, STOP — do not "fix it up" or continue.
   Report what you got.
5. Obey the **DO NOT** block. Those are the mistakes that break the build or
   wander out of scope.
6. **Commit** with the exact message given.

Never make an architectural decision. If a task seems to require choosing between
two designs, it is a bug in the task — stop and report it.

## Task status legend

Each task and stage carries a status. Be honest about it:

- **UNVERIFIED** — authored but never executed on a real machine. Most VM tasks
  start here. Do not describe an UNVERIFIED task as working.
- **VERIFIED** — executed; the acceptance command passed and the transcript is
  recorded in the matching `docs/qa/stage-N.md`.
- **BLOCKED** — cannot proceed; the reason is written in the task.

When you finish a task and its acceptance passes, change its status to VERIFIED
and paste the transcript into the QA doc.

## Task template (every task follows this)

```text
### Task N.M — <title>   [STATUS]
Precondition: <a command whose success proves you're ready>
Files: Create/Modify <exact repo-relative paths>
Signature: <exact fn/struct/CLI signature, if code>
Steps:
  1. <one concrete action>
  2. <one concrete action>
Acceptance:
  $ <command>
  → expect: <exact stdout / exit 0 / screenshot description>
DO NOT:
  - <scope/build trap 1>
  - <scope/build trap 2>
Commit: <conventional-commit message>
```

## A note on the VM tasks

Stage 0's VM tasks are **UNVERIFIED** until someone runs them on a real UTM VM.
aarch64 Arch package names and boot specifics can differ from x86 Arch; where a
value must be confirmed at runtime, the task says **CONFIRM AT RUNTIME** and
tells you how. Do not guess — run the check the task gives you.
