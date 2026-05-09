## Goal

Read the project pulse with one command.

## Why

`aida status` is the most informative single command in AIDA. It
pulls together everything important about a project's state:

- **Project** — name, mode, store path
- **Requirements** — count by status (Approved/InProgress/Completed/...)
- **Cache** — freshness (FRESH = good; STALE/MISSING = run
  `aida cache rebuild`)
- **Sync** — orphan-store ahead/behind origin (with the right command
  to fix)
- **Recent activity** — last 5 user-authored req changes (META rows
  filtered out)
- **Scaffolding** — whether AIDA-owned files match embedded templates,
  separately surfaced from user customizations to CLAUDE.md/AGENTS.md

This is the command you run at the start of a session: "where am I,
what changed since last time, what needs my attention."

Inside Claude Code, the `/aida-status` slash command runs the same
output and helps the model orient before you give your next instruction.

## What to do

From `workspace/`, run `aida status`. Read all the sections.

## Tip

`aida status` ends with a "AIDA development context" section ONLY when
you're inside the AIDA repo itself (it shows binary version, release
readiness, template symlink health). For your own projects that section
is hidden by default — pass `--no-dev-context` to suppress it explicitly,
or it just won't appear.

## Verify

Read-only. `aida-tutor verify` passes once prerequisites exist.
