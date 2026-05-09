## Goal

Read the audit trail on FR-1 — title, status, description, **and** the
comment you just added.

## Why

`aida show <id>` prints the record header and description. Adding
`--comments` extends the output with the full comment thread:

```
Comments:

019e0e74-7023-...:
  By: joe at 2026-05-09 15:01
  v0 stub committed (sha 28611f2). Real wiring per ADR-1 still TODO.
```

Three things make this useful:

1. **Author + timestamp on every comment** — when something changed
   and who said what.
2. **`(edited 2026-05-09 15:30)`** appears when a comment has been
   modified after creation, so you can see history evolving.
3. **Threaded replies** — comments support parent/child relations,
   surfaced as indented nesting.

This is the "what survived in writing?" view. Useful when reviewing
work, onboarding, or constructing a release-notes draft.

## What to do

From `workspace/`, run:

```bash
aida show <FR-id> --comments
```

Read what comes back. Verify your comment from exercise 14 is there
with author/timestamp/content.

## Tip

For a denser view of just the comments (no header), use `aida comment
list <id>` — same data, less ceremony.

## Verify

Read-only. `aida-tutor verify` passes on prerequisite state.
