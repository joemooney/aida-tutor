## Goal

Finish the task with `aida queue done` — and watch it close the
requirement *and* leave the queue in one atomic step.

## Why

When you finish a queued item, two things must happen:

- The requirement's status moves to done.
- The item leaves the queue.

Do those as two separate commands and you can get them out of sync —
forget the second and a finished task lingers in the queue; forget the
first and a dequeued task is left half-open. Either way the queue stops
telling the truth about what's left.

`aida queue done <ID>` does both as **one atomic step**. It's exactly
equivalent to:

```
aida edit <ID> --status completed
aida queue remove <ID>
```

…but you can't run only half of it. That's the point — the queue is
only useful if "in the queue" reliably means "still needs doing."

This closes the producer/consumer loop: routed (22) → claimed (23) →
done (24).

## What to do

From `workspace/`, finish the task you've been carrying. `aida queue
done` asks for confirmation; `--yes` skips the prompt:

```
aida queue done TASK-3 --yes
```

(Use the real ID.) `aida` confirms it's marked done and removed from
the queue. Run `aida queue next` afterward — your queue is empty.

## Tip

In the implementer loop you'd immediately `aida queue next` again and
pick up whatever's below — done-then-pickup is the rhythm. Inside
Claude Code, finishing with `/aida-pickup`'s `aida queue done` and
grabbing the next item is a single prompt.

## Verify

`aida-tutor verify` — passes once the "Queue demo" task is done **and**
no longer in any queue. If it's done but still queued, the atomic step
didn't hold — the verifier will tell you.

## What's next

That's the roles + queue cluster. You've worn a hat, routed work as the
producer, and picked it up and closed it as the consumer. Run
`aida-tutor progress` to see where you stand.
