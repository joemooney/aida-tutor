<!-- trace:STORY-60 | ai:codex -->

# See the work before coordinating it

The operator loop answers four questions: what is happening, what needs a
human, what is actually alive, and whether the project is healthy.

```bash
aida status --no-ci
aida awaiting --no-ci
aida ps
aida health --brief
```

For multiple agents or clones, inspect the surrounding coordination state:

```bash
aida mailbox list
aida team
aida node whoami
aida worktree list
```

These are read-only views. Inspect first; only then assign, enter a worktree,
or launch work.
