## Goal

Delete the SQLite cache, rebuild it from the git store with `aida cache
rebuild`, and confirm nothing was lost.

## Why

`.aida/cache.db` is a SQLite *projection* of the orphan branch — a fast
read index for `aida list` and `aida search`. It is **not** canonical.
It's gitignored, it's never pushed, and it can always be regenerated
from the `aida-store` branch.

Knowing this changes how you treat it:

- A fresh `git clone` arrives with **no** cache — the first `aida`
  command that needs it just rebuilds it.
- If the cache ever drifts or gets corrupted, you don't debug it. You
  delete it and rebuild.
- You never resolve a merge conflict in it, never back it up, never
  commit it.

The orphan branch is the source of truth. The cache is a throwaway.
This exercise makes you *do* the throwaway-and-rebuild so it stops
feeling scary.

## What to do

From `workspace/`, delete the cache:

```
rm .aida/cache.db
```

Then rebuild it from the git store:

```
aida cache rebuild
```

It prints how many requirements it projected from the orphan branch.
Confirm the read path works again:

```
aida list
```

Everything is back — because nothing was ever *in* the cache that
wasn't already on the `aida-store` branch.

## Tip

`aida cache rebuild` rebuilds from scratch; `aida cache status` tells
you whether a rebuild is even needed (it reports `STALE` when the
cache HEAD lags the store HEAD). Most of the time you can ignore the
cache entirely — AIDA keeps it fresh for you.

## Verify

`aida-tutor verify` — passes once `.aida/cache.db` exists again and is
a valid SQLite database. (A corrupted, non-rebuilt cache fails this
check — so finish the rebuild.)

## What's next

That's the distributed-storage cluster. You've seen the orphan branch,
its live history, the two-leg push, and the disposable cache. Run
`aida-tutor progress` to see where you stand.
