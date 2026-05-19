## Goal

Audit the requirement store's health with `aida db info` and
`aida db check`.

## Why

You've trusted the store for 33 exercises. This one shows you how to
*check* it.

The store is not a black box — it's plain YAML files committed to the
orphan `aida-store` branch (you saw them in exercise 18). Plain files
can be inspected, and AIDA gives you two commands to do it:

- **`aida db info`** — the stat view. Storage backend, the store's
  on-disk path, how many requirements it holds, the agreed-id blocks
  (the pre-allocated id ranges each type draws from), and whether the
  store worktree has uncommitted changes.
- **`aida db check --collisions`** — the fsck. It scans every
  requirement for a *collision*: two requirements claiming the same
  display id. The merge gate prevents new collisions, but a store that
  predates that check can still carry one — so the audit stays useful.

The cache is disposable (exercise 20). The store is canonical. Knowing
how to audit the canonical thing is what lets you trust it.

## What to do

From `workspace/`, look at the store's vital statistics:

```
aida db info
```

Read the output: the backend is `Git (sharded YAML)`, the path points
at `.aida-store`, and the requirement count matches what `aida list`
shows. Then run the consistency check:

```
aida db check --collisions
```

A healthy store reports `✓ No agreed-id collisions found`. Your
tutorial store is healthy, so that's what you'll see — but now you know
the command for the day a real store isn't.

## Verify

`aida-tutor verify` — passes once the store is collision-free. The
verifier runs the same scan `aida db check --collisions` does: it walks
every requirement file and confirms no two claim the same id.

## Tip

`aida db check --collisions --repair` doesn't just report a collision —
it re-gates the later claimant onto the next free id and keeps the
earlier one. And `aida db status` (distinct from `db info`) reports the
orphan store's sync state: local changes, ahead/behind the remote, and
any unresolved conflicts.

## What's next

Exercise 35 — the last one — connects AIDA to an AI agent with
`aida mcp-serve`.
