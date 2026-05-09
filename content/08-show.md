## Goal

Read one requirement in full.

## Why

`aida list` shows the row; `aida show <id>` shows the full record —
title, status, priority, the description rendered as readable prose,
the UUID, and (with `--comments`) any audit-trail comments.

This is the command you reach for when:

- A teammate references "FR-12" and you want context
- You're picking up an in-progress req and need to remember what's in
  scope
- An agent (Claude, Codex) needs to be filled in on a spec before
  implementing

## What to do

From `workspace/`, run `aida show <id>` — pick any of your reqs
(e.g. `aida show FR-1` or `aida show VIS-1`). Read the output.

Optionally, try `aida show <id> --comments`. Comments will be empty
right now; we'll add some in exercise 14.

## Tip

`aida show` accepts spec-ids (FR-1) AND uuids. The spec-id form is
shorter and human-readable; the uuid form survives merges between
projects. Internally everything is uuid — the spec-id is a label.

## Verify

Read-only command. `aida-tutor verify` passes once the prerequisite
state from earlier exercises exists.
