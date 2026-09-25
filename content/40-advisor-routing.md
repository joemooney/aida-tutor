<!-- trace:STORY-59 | ai:codex -->

# Route uncertainty instead of guessing

When implementation hits a real design fork, pause the spec rather than
inventing authority. `aida punt` records the category and reason in Needs
Attention. The decision inbox makes the handoff visible:

```bash
aida punt TASK-ID --category design-fork --reason "Two valid designs need a human choice."
aida questions list
aida groom --dry-run
```

`groom --dry-run` is propose-only. It lets an advisor inspect draft intake
without silently approving or queueing work. A SPIKE can use `aida research
SPIKE-ID --dry-run` to preview the evidence-gathering prompt before dispatch.
