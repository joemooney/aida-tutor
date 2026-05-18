## Goal

Capture a requirement, then watch it land as a real commit on the
`aida-store` orphan branch — and see how `aida push` ships that branch
alongside your code.

## Why

Exercise 18 showed you the orphan branch exists. This one shows you
it's *live*: every `aida add` and `aida edit` auto-commits to it.

That's the whole point of git-canonical mode. You don't "save" the
database — each mutation is already a git commit on `aida-store`. Run
`git log aida-store` and you'll see one commit per capture, with a
real SHA and a real history you can diff, blame, and revert.

Because the store is just a branch, sharing it is just `git push` —
but on a *different* branch than your code. That's the classic split:

- `git push` ships your **code** branch.
- The store needs `aida-store` pushed too.

Do one and forget the other and your teammate gets the code without
the requirements, or vice versa. `aida push` runs both legs in one
command so they never drift apart.

## What to do

From `workspace/`, capture anything:

```
aida add --type functional --status approved --title "your idea here"
```

Then look at the orphan branch's history:

```
git log aida-store --oneline
```

Your capture is the top commit. Now inspect the pairing and the push
plan:

```
aida store status
aida push --dry-run
```

`aida store status` shows your code HEAD and the store SHA it's paired
with. `aida push --dry-run` shows both legs without pushing — in this
tutorial there's no `origin`, so both legs report "nothing to push",
but the shape is what a real two-leg push looks like.

## Tip

`aida db sync --push` is the store-only leg on its own; `aida push
--store-only` is the same thing via the unified command.

## Verify

`aida-tutor verify` — passes once the `aida-store` branch carries
commits beyond `aida init`'s bootstrap (i.e. at least one of your
captures has landed on it).

## What's next

Exercise 20 — delete the cache and rebuild it, proving the orphan
branch is the only thing that's canonical.
