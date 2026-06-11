## Goal

Capture your first piece of work — a thing the system should do — with
`aida add`.

## Why

Capturing is the whole front door of AIDA. The simplest possible form is
just free text:

```bash
aida add "Add login validation"
```

That one line records a task (status `draft`) and hands you back an id.
No flags, no jargon — if all you ever learn is `aida add "..."` and
`aida done <id>` (exercise 08), you already have a working loop.

For this tutorial we'll capture a slightly richer requirement — a
**functional requirement** (a behavior, prefix `FR`) — because the next
several exercises build on it: you'll write code for it, link that code
back to it, and watch the link appear. Marking it `approved` says "yes,
we intend to build this."

## What to do

From `workspace/`, capture the running example. The title can be
positional (the newcomer form) with a couple of flags to type and approve
it:

```bash
aida add "Convert timestamps between time zones" --type functional --status approved --priority high
```

Use any behavior you like for the title — phrase it as a verb-y action
("Parse JSON input", "Persist user preferences"). It becomes **FR-1**, the
spec we follow through the rest of the core loop.

## Tip

`--title "..."` is the long form of the positional title; if you pass
both, `--title` wins. A description is optional here — `aida edit FR-1
--description "..."` can flesh it out later.

## Verify

`aida-tutor verify` — looks for any `FR-*` requirement with status approved.
