## Goal

Reopen FR-1 and move it back to `in-progress` — and meet AIDA's guard on
closed requirements.

## Why

Work isn't always one-and-done. You closed FR-1 in exercise 08, but
suppose a follow-up lands and you need to pick it back up. The
`in-progress` status is the "actively working on this" signal, and it
underpins a few useful behaviors:

1. **`aida list --status in-progress`** — shows only what's actively
   being worked on.
2. **`/aida-pickup`** (in Claude Code) — pulls the next requirement off
   your work queue and flips it to `in-progress` automatically.
3. **Recent activity in `aida status`** — status flips are a major
   source of "what changed recently".

Because reopening a *closed* requirement is usually a mistake, AIDA
guards it: a plain `aida edit FR-1 --status in-progress` will refuse and
tell you to pass `--force`. That guardrail is the lesson here.

## What to do

From `workspace/`, try the plain edit first to see the guard, then force
it:

```bash
aida edit FR-1 --status in-progress          # refused — FR-1 is Completed
aida edit FR-1 --status in-progress --force  # reopened
```

## Tip

Status names accept a few variants — `in-progress`, `inprogress`,
`in_progress` all work. The CLI normalizes them. The full status
vocabulary (`approved`, `planned`, `in-progress`, …) lives behind `aida
edit --status`; `aida done` from exercise 08 is just the friendly
shortcut for the common "completed" case.

## Verify

`aida-tutor verify` — checks any `FR-*` is now in-progress.
