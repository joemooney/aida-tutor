## Goal

Step into the session worktree, do real work there, and commit it — and
see that the commit lands on the session branch, not the parent's.

## Why

The whole point of a worktree is **isolation**. The session worktree is
a full, ordinary checkout: you `cd` into it and use git exactly as
always. But it's on its own branch (`session-work`), so:

- Commits you make there advance `session-work`. The branch your
  `workspace/` sits on never moves.
- Your `workspace/` working tree is undisturbed — no stashing, no
  "finish this before you can touch that."

Two things are *shared*, on purpose:

- **The `.git` repository.** Both checkouts are the same repo, so the
  `session-work` branch and its commits are visible from `workspace/`
  too (`git log session-work`) — they're just not *on* the parent's
  branch.
- **The AIDA store.** `.aida-store/` is symlinked, so a requirement
  edited from either side is the same requirement. Code branches;
  requirements don't.

That split — diverge the code, share the requirements — is what makes a
session safe to run alongside other work.

## What to do

`cd` into the worktree `aida session start` created (its path was in the
command's output — a sibling of `workspace/`). Then do some work and
commit it. For example:

```
cd ../workspace-fr-1
mkdir -p src
echo '// trace:FR-1 | ai:claude' > src/widget.rs
echo 'pub fn widget() {}' >> src/widget.rs
git add src/widget.rs
git commit -m "[AI:claude] feat(widget): build widget (FR-1)"
```

Then look at the two branches — same repo, different tips:

```
git log --oneline session-work
git log --oneline master
```

Your commit is on `session-work` only.

## Verify

`aida-tutor verify` — passes once `session-work` has at least one commit
the base branch doesn't. Run it from `workspace/` or the tutor root.

## Tip

`aida` commands run from the worktree only if `.aida/config.toml` is
committed to the repo (it's git-tracked but optional). Requirement work
is easiest from `workspace/`; the store is shared either way.

## What's next

Exercise 29 — list the active session leases and inspect yours.
