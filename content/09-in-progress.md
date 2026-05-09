## Goal

Move FR-1 from `approved` to `in-progress`.

## Why

Status changes are how AIDA tracks work-in-flight. Today it's just a
field. But it underpins three useful behaviors:

1. **`aida list --status in-progress`** — shows only what's actively
   being worked on. Useful when several teammates are picking
   different tickets.
2. **`/aida-pickup`** (in Claude Code) — picks the highest-priority
   `approved` req and flips it to `in-progress` automatically. Status
   is the signal AIDA uses to know what's queued vs in-flight.
3. **Recent activity in `aida status`** — surfaces what changed
   recently. Status flips are a major source of "recent activity".

## What to do

From `workspace/`, run `aida edit <FR-id> --status in-progress`.

Replace `<FR-id>` with the actual id of your FR (from exercise 5).
Probably `FR-1`.

## Tip

Status names accept a few variants — `in-progress`, `inprogress`,
`in_progress` all work. The CLI normalizes them.

## Verify

`aida-tutor verify` — checks any `FR-*` is now in-progress.
