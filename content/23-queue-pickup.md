## Goal

Pick up the task you queued: peek at the head of your role's queue,
then claim it.

## Why

You're the consumer now. In exercise 22 the producer routed a task to
the implementer queue. Wearing the `implementer` hat (exercise 21), you
pull from that queue.

Picking up an item is two moves:

1. **Peek.** `aida queue next` shows the top item routed to your active
   role — its ID, title, the producer's note — *without* removing it.
   Peeking is safe; it doesn't claim anything.
2. **Claim.** `aida edit <ID> --status in-progress` flips the
   requirement's status. That's the signal to everyone else — other
   sessions, dashboards — that someone is on it.

Inside Claude Code you'd rarely type those by hand: the `/aida-pickup`
slash command does the whole sequence — peek the head, mark it
in-progress, print the spec card — in one step. But `/aida-pickup` is
just a wrapper. The moving parts are `queue next` and `edit`, and this
exercise is those parts.

## What to do

From `workspace/`, with the `implementer` role active, peek at the
queue:

```
aida queue next
```

It shows the "Queue demo" task you routed in exercise 22, and even
suggests the claim command. Run it (with the real ID):

```
aida edit TASK-3 --status in-progress
```

## Tip

`aida queue next` filters by your **active role**. If it says the queue
is empty, check `aida statusline` — you may have ended the role, or
never entered it (exercise 21). The queue didn't lose your task; you're
just looking at it through the wrong hat.

## Verify

`aida-tutor verify` — passes once the "Queue demo" task is
`in-progress`. Peeking leaves no trace, so the claim — the status flip
— is what the verifier checks.

## What's next

Exercise 24 — finish the task and clear it from the queue in one move.
