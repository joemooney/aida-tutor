## Goal

End the session: remove the worktree, release the lease — and confirm
your branch and its commits survive.

## Why

A session is meant to be torn down when its work is parked or finished.
`aida session end <id>` does the cleanup:

- **Removes the git worktree.** The sibling directory is deleted.
- **Deletes the lease.** `.aida/sessions/<id>.toml` is gone, so the
  scope is free again and `aida session leases` no longer lists it.
- **Keeps the branch.** This is the deliberate part. `session-work`
  and the commit you made in exercise 28 are *not* touched. Ending a
  session is about releasing the *room*, not the *work* — you merge or
  discard the branch on your own schedule.

That separation matters: tearing down a worktree should never be the
thing that loses your commits.

## What to do

From `workspace/`, find the session id (`aida session leases` if you
don't have it), then end it. `--yes` skips the confirmation:

```
aida session end <id> --yes
```

(Use the real id.) Now confirm the three outcomes:

```
git worktree list      # the session worktree is gone
aida session leases    # no active leases
git branch             # session-work is still here
git log --oneline session-work
```

The worktree and lease are gone; the branch and its commit remain.

## Verify

`aida-tutor verify` — passes once the `session-work` worktree and lease
are gone *and* the `session-work` branch still exists.

## Tip

`aida session end` refuses to remove a worktree with uncommitted changes
(or a live `claude`) inside — a guard against losing work. `--force`
overrides it; you committed in exercise 28, so you don't need it here.

## What's next

That closes the sessions + worktrees cluster. Exercise 31 opens the
last one — code review and commit pairing: how AIDA pins every commit
to a store version, and turns requirements into review briefs.
