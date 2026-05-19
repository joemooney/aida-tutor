## Goal

Read the project pulse with one command.

## Why

`aida status` is the most informative single command in AIDA. It
answers "where am I, what changed, what needs my attention" in one
screen. Depending on what's set up, you'll see some or all of:

- **Session** — the active scoped session covering this directory
  (id, scope, role, worktree) — covered in exercises 27-30
- **Branch** — current branch, dirty state, ahead/behind origin
- **PR / CI** — open pull request for the branch + check status
- **Queue** — items routed to your active role — covered in 21-24
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

Each section graceful-degrades: when there's nothing to report (no
session yet, no open PR, empty queue) it stays quiet. Early in this
tutorial you'll see the bottom half of that list — the session and
queue sections fill in once you reach those exercises.

Inside Claude Code, the `/aida-status` slash command runs the same
output and helps the model orient before you give your next instruction.

## What to do

From `workspace/`, run `aida status`. Read all the sections.

## Tip

`aida status --short` collapses everything into a one-line readout —
role, scope, branch — for a quick "where am I" without the full screen.
`--queue` and `--ci` zoom in on a single section, and `--json` emits
the whole thing machine-readably.

## Verify

Read-only. `aida-tutor verify` passes once prerequisites exist.
