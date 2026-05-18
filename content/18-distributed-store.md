## Goal

Look inside the distributed store that `aida init` built — the orphan
branch, the linked worktree, and the rebuildable cache.

## Why

Back in exercise 01 you ran `aida init` and three things appeared. Now
you'll see *what* they are, because understanding the layout is what
makes the rest of distributed mode make sense.

AIDA's default storage is **git-canonical and distributed**: your
requirements live in git itself, not in a database file you have to
back up separately. Three pieces make that work:

1. **The `aida-store` orphan branch.** A branch with no shared history
   with your code branches — its own root commit, its own log. Every
   requirement, comment, and relationship is a file committed there.
2. **The `.aida-store/` linked worktree.** That orphan branch is checked
   out as a *worktree* at `.aida-store/`, so you can see the YAML files
   on disk while still working on your code branch in the main tree.
   (`.aida-store/.git` is a small pointer file, not a directory — that's
   the tell-tale sign of a linked worktree.)
3. **The `.aida/cache.db` SQLite cache.** A fast read index projected
   *from* the orphan branch. It's gitignored and disposable — exercise
   20 shows you rebuilding it from scratch.

The canonical data is the orphan branch. The cache is just for speed.

## What to do

From `workspace/`, run these three read-only commands and read the
output:

```
git branch
git worktree list
aida cache status
```

`git branch` shows `aida-store` alongside your code branch. `git
worktree list` shows `.aida-store` checked out on it. `aida cache
status` prints the cache path, its requirement count, and how its
HEAD compares to the store HEAD (`FRESH` means in sync).

`aida db path` is a handy fourth: it prints the active store location.

## Verify

`aida-tutor verify` — passes once it sees the `aida-store` branch, the
`.aida-store/` linked worktree, and a valid `.aida/cache.db`. If the
cache hasn't materialized yet, run `aida cache status` once to create
it.

## What's next

Exercise 19 — watch a capture become a commit on that orphan branch.
