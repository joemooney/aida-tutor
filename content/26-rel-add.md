## Goal

Link two requirements that already exist with `aida rel add`, using a
*typed* relationship and the `--bidirectional` flag.

## Why

Exercise 25 created a relationship at capture time with `--parent`.
But you often need to link requirements that *already exist* — and
with an edge that isn't parent/child.

`aida rel add` does that. It takes two requirements and a `--type`:

```
aida rel add <FROM> <TO> --type <kind>
```

AIDA knows several relationship kinds. Two useful ones:

- **`verifies`** — `<FROM>` proves `<TO>` works: a test or QA task
  verifies a feature. Its inverse is `verified-by`.
- **`references`** — a looser "see also" link. Its inverse is just
  `references` again.

By default `aida rel add` writes the edge on the **source only**. Pass
**`--bidirectional`** and AIDA also writes the **inverse** edge on the
target — so `Verifies` on one end becomes `VerifiedBy` on the other.
That's the same two-sided bookkeeping `--parent` did for you in
exercise 25, and it's what keeps the graph walkable from both ends.

## What to do

From `workspace/`, capture the two requirements to link — a feature
and the task that will verify it. Use these exact titles:

```
aida add --type functional --status approved --title "Relationships demo: feature under test"
aida add --type task --status approved --title "Relationships demo: verifying task"
```

Note both IDs (an `FR-…` and a `TASK-…`). Now link them — the task
*verifies* the feature — and pass `--bidirectional` so both ends get
the edge:

```
aida rel add TASK-9 FR-8 --type verifies --bidirectional
```

(Substitute the real IDs.) `aida` confirms both legs: the relationship
`TASK-9 --[Verifies]--> FR-8` and the inverse
`FR-8 --[VerifiedBy]--> TASK-9`.

## Tip

`aida rel add` also accepts the flag form — `aida rel add --from
TASK-9 --to FR-8 --type verifies` — if you find the positional order
hard to remember. `aida rel list` shows every edge in the store;
`aida rel remove` deletes one.

## Verify

`aida-tutor verify` — passes once the task has a `Verifies` edge to
the feature *and* the feature has the inverse `VerifiedBy` edge back.
If only the first half is there, you left off `--bidirectional`.

## What's next

That's the relationships cluster — and the last exercise for now. Run
`aida-tutor progress` to see the full board. You've gone from `aida
init` through capture, trace, commit, distributed storage, roles +
queue, and the requirement graph itself.
