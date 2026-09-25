## That's the tour

In about fifteen minutes you captured an intent, had it built, and watched
a memoryless agent answer from the project's own record. That round trip —
spec to commit to code, readable by any agent — is the core of AIDA, and
you now have the one idea the rest of it builds on. When you want the
guided depth — the distributed store, search, the full command surface —
the 36-exercise track is one command away: `aida-tutor list`.

## What's next — production automation

This tour used the smallest reliable handoff: capture a spec, run a
readiness check, and let the selected AI implementer edit the scratch file.
On a real project, the same idea scales up:

```
aida zen FR-1 --vendor codex --dry-run
aida zen FR-1 --vendor codex
```

`aida zen` is the one-spec autonomous path: implement, review, merge, and
pull when the repo is ready for that level of automation. For a batch of
advisor-approved work, queue the ready specs and inspect the fan-out plan:

```
aida burndown plan --candidates
aida burndown plan
```

The full burndown run is the higher-trust path. Use it after the advisor
has blessed a queue and your repo has the review/CI expectations you want
the agents to satisfy.
