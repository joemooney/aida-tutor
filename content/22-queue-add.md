## Goal

Capture a task and route it into a role's work queue with
`aida queue add --for`.

## Why

The **queue** is AIDA's producer/consumer channel. One person (or one
session) decides *what* needs doing and routes it; another picks it up
and does it. The two halves are deliberately separate:

- The **producer** wears the `dialog` hat — chatting, capturing
  requirements, deciding priorities — and routes work to a doer role
  with `aida queue add <ID> --for <role>`.
- The **consumer** wears `implementer` (or `reviewer`, …) and pulls
  from *their* queue with `aida queue next` (exercise 23).

`--for` is the routing label. `aida queue add FR-1 --for implementer`
doesn't add to *your* pile — it adds to the **implementer queue**,
wherever that work eventually gets picked up. That's how a planning
session hands work to a coding session without either one micromanaging
the other.

This exercise is the producer side: capture one task, route it.

## What to do

From `workspace/`, capture the task this cluster will follow. Use this
exact title — exercises 23 and 24 track the same task by it:

```
aida add --type task --status approved --title "Queue demo: ship the sample widget"
```

Note the ID it prints (something like `TASK-3`). Route that ID to the
implementer queue:

```
aida queue add TASK-3 --for implementer
```

(Substitute the real ID.) `aida` confirms with `[for:implementer]`.

## Tip

`aida queue add` also takes `--note "..."` — a line explaining *why*
this was queued. The consumer sees it in `aida queue next`, so a good
note ("customer ask, blocks the release") saves a round-trip.

## Verify

`aida-tutor verify` — passes once the "Queue demo" task exists and a
queue entry routes it to a role (`for_role` set). Both halves matter:
the task captured, and the task routed.

## What's next

Exercise 23 — switch to the consumer side and pick the task back up.
