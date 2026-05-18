## Goal

Start a *scoped session* — claim a slice of work and give it its own
git worktree on its own branch.

## Why

So far you've worked everything in one directory, on one branch. That's
fine for a solo, one-thing-at-a-time pace. It stops being fine the
moment two pieces of work overlap: a half-finished feature on the branch
when an urgent fix comes in, or two agents wanting the same repo at once.

A **session** solves this. `aida session start` does three things in one
command:

1. **Forks a git worktree.** A worktree is a *second checkout* of the
   same repo — its own directory, its own branch — sharing the one
   `.git`. Your `workspace/` stays exactly as it is; the session gets a
   sibling directory to work in.
2. **Symlinks the AIDA store into it.** The worktree's `.aida-store/` is
   a symlink back to the real one. Requirements stay shared — both
   checkouts see the same store — even though the code branches diverge.
3. **Writes a lease.** A small file at `.aida/sessions/<id>.toml`
   recording who owns what scope, on which branch, in which worktree.
   That's how concurrent sessions stay out of each other's way.

The *scope* is what the session owns: an EPIC, a spec id, a path glob,
or a free-form tag. It's advisory — a coordination signal, not a lock.

## What to do

From `workspace/`, scope a session to the feature you captured back in
exercise 05. Confirm its id first:

```
aida list --type functional
```

It's `FR-1`. Now start the session — pin the branch name to
`session-work` so the rest of this cluster can find it:

```
aida session start --owns FR-1 --branch session-work
```

`aida` prints the new worktree's path (a sibling of `workspace/`, named
for the scope), the branch, and the lease file. Note that path — the
next exercise works inside it.

## Verify

`aida-tutor verify` — passes once a `session-work` worktree exists and
its lease is on disk.

## Tip

You didn't have to name the branch — without `--branch`, `aida` derives
one from the scope. We pin it here only so exercises 28-30 can follow
this exact session.

## What's next

Exercise 28 — `cd` into the worktree and do work there.
