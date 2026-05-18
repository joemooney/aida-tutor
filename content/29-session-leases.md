## Goal

See the active sessions — the "who holds what right now" view — and
drill into one.

## Why

A lease is only useful if you can read it. When one session runs, the
lease list is a curiosity. When three run at once — yours, a reviewer's,
an agent draining a queue — it's the coordination surface: before you
start work, you glance at it to see whether someone already owns that
scope.

Two read-only commands:

- **`aida session leases`** — one row per active session: id, scope,
  branch, role, worktree. The canonical "what's running?" view.
- **`aida session show <id>`** — drills into one lease: its worktree
  path, the lease file, recent activity in that session, and whether a
  live `claude` process is inside the worktree.

Neither changes anything. They inspect — the same way `aida list` and
`aida show` inspect requirements.

Don't confuse `aida session leases` with `aida session list`: `list`
shows *historical Claude Code conversations* in the project; `leases`
shows the *scoped worktrees* `aida session start` created. For "what's
running?" you want `leases`.

## What to do

From `workspace/`, list the leases:

```
aida session leases
```

You'll see the `session-work` session from exercise 27. Copy its id,
then drill in:

```
aida session show <id>
```

(An 8-character id prefix is enough.) Note the worktree path, the lease
file location, and the activity section.

## Verify

`aida-tutor verify` — passes while the `session-work` session is still
active. (These commands are read-only, so the tutor verifies the
prerequisite: a session for you to inspect.)

## Tip

`aida session leases --verbose` adds the live-`claude` PID and flags
worktrees whose process outlived their directory — handy for spotting a
session that ended badly.

## What's next

Exercise 30 — end the session and release the worktree.
