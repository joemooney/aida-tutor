## Goal

Run `aida push` — the unified code+store push.

## Why

In a normal project there are two things to push:

1. Your code changes (current branch on `origin`)
2. The orphan AIDA store (`aida-store` branch on `origin`)

The classic fail mode is doing one but forgetting the other — code
gets shared, but the requirements you wrote stay local. Or vice versa.

`aida push` runs both in one shot. With no `origin` configured (which
is the case for this tutorial), both legs gracefully skip with a Note:

```
  Note: no `origin` remote — skipping code push
  Note: orphan store has no `origin` — skipping store push
```

Once you add `origin` (e.g. `git remote add origin <url>`), the same
command does the actual two pushes.

## What to do

From `workspace/`, run `aida push`.

You'll see the two skip Notes. That's the expected output for a no-
remote project. Run it anyway — you've now seen the command's output
both with and without origin (it's identical in shape, just different
lines below the Notes).

## Tip

`aida push --code-only` skips the store leg; `aida push --store-only`
skips the code leg. Useful when one side is in a state you don't want
to push yet.

## Verify

`aida-tutor verify` — passes once prerequisites exist (the push
itself doesn't change state we can detect).

## What's next

Run `aida-tutor progress` to see your final stats. You've walked the
full AIDA loop: init, capture, edit, trace, commit, document, search,
status, push.

From here, the model in your editor (Claude Code or Codex) can read
the AIDA conventions out of `.claude/AIDA.md` and `AGENTS.md` and
work alongside you using the same vocabulary you just learned.
