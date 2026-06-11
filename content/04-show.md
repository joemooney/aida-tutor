## Goal

Read FR-1 in full — and notice what's *not* there yet.

## Why

`aida list` shows the row; `aida show <id>` shows the full record —
title, status, priority, the description as readable prose, the UUID, and
a **Git linkage** section that connects the spec to the code that
implements it.

This is the command you reach for when:

- A teammate references "FR-1" and you want context
- An agent (Claude, Codex) needs to be filled in on a spec before
  implementing
- You want to know whether a spec is actually built yet — and where

## What to do

From `workspace/`, run:

```bash
aida show FR-1
```

Read the output. Look at the **Git linkage** section near the bottom —
right now it's empty (or absent): nothing in the repo points at FR-1 yet.

**Remember that.** In the next three exercises you'll write code that
references FR-1, commit it, and come back to this exact command in
exercise 07 to watch the linkage fill itself in. That's the payoff the
whole core loop is building toward.

## Tip

`aida show` accepts spec-ids (FR-1) and uuids. The spec-id form is shorter
and human-readable; the uuid form survives merges between projects.
Internally everything is uuid — the spec-id is just a label.

## Verify

Read-only command. `aida-tutor verify` passes once FR-1 exists (exercise
02).
