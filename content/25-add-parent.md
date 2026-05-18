## Goal

Capture a requirement *underneath* another one — a parent/child
relationship — with `aida add --parent`.

## Why

Requirements aren't a flat list. A broad **epic** ("v2 onboarding
flow") breaks down into **stories**; a story breaks down into
**tasks**. AIDA models that with a **relationship** — a typed edge
between two requirements — and parent/child is the most common kind.

The fastest way to create one is at capture time. `aida add --parent
<ID>` captures the new requirement *and* links it under `<ID>` in a
single step — no separate linking command.

A relationship in AIDA is always written on **both** endpoints:

- the child gets a `Child` edge pointing at the parent, and
- the parent gets the inverse `Parent` edge pointing back.

You only typed one `--parent`; AIDA wrote both sides. That two-sided
bookkeeping is what keeps the graph consistent — you can walk it from
either end (`aida show` on the parent lists its children; on the child
it shows the parent).

## What to do

From `workspace/`, first capture an epic to be the umbrella. Use this
exact title — the verifier looks for it:

```
aida add --type epic --status approved --title "Relationships demo: umbrella epic"
```

Note the ID it prints (something like `EPIC-7`). Now capture a story
*under* that epic with `--parent`:

```
aida add --parent EPIC-7 --type story --status approved --title "Relationships demo: child story"
```

(Substitute the real epic ID.) `aida` confirms the link with a line
like `Linked: EPIC-7 → parent of STORY-8`.

## Tip

`--parent` isn't only for epic→story. Any requirement can parent any
other — a story can parent its tasks, a bug can parent the follow-up
tasks that fix it. And if you forget `--parent` at capture time,
exercise 26's `aida rel add` links two requirements after the fact.

## Verify

`aida-tutor verify` — passes once the epic and the story both exist
*and* the parent/child edge is on both ends: the story's `Child` edge
to the epic, and the epic's inverse `Parent` edge back.

## What's next

Exercise 26 — link two requirements that already exist, with a typed
relationship and `aida rel add`.
