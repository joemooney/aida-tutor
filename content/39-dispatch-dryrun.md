<!-- trace:STORY-58 | ai:codex -->

# One-verb workflows: preview before side effects

AIDA now makes workflow contracts discoverable. `aida do` dispatches a spec
according to its groomed execution mode; `aida zen` is the autonomous
implement-and-ship path; `aida ship` is the human-implementer finish path.

Preview them from the tutorial workspace:

```bash
aida do FR-1 --mode operator
aida zen FR-1 --dry-run
aida ship FR-1 --dry-run
```

Read the suitability gate and the human checkpoint in each output. The
tutorial only uses dry runs: never launch a real agent or ship a PR from the
exercise sandbox.
