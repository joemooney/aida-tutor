## Goal

Make an ordinary `git commit` and watch AIDA pin it to the store — an
`Aida-Store: <sha>` trailer that records exactly which version of the
requirement store the commit was written against.

## Why

Your code and your AIDA store live in the *same* repo but evolve on
*different* branches — the code on `main` (and feature branches), the
store on the orphan `aida-store` branch you met in exercises 18–20.
They move independently.

That independence has a cost. Check out a code commit from six months
ago and its `// trace:FR-7` comments point into *today's* store — where
`FR-7` may have been renamed, re-scoped, or deleted. The trace dangles.
The commit no longer knows which store it was written against.

AIDA fixes this at commit time. Back in exercise 01, `aida init`
installed a **`prepare-commit-msg` hook**. On every commit it reads the
orphan store's current HEAD and appends a trailer:

```
Aida-Store: 48892b2d85e5d5d8e2a4be032250c679785e7fb2
```

Now the commit is *paired* with a store SHA. `aida store status` reads
that trailer back and tells you whether your checkout's code and store
are aligned or have drifted apart. The hook degrades quietly — no
orphan store, no remote — so it never fails a commit.

You don't enable anything. The hook is already there. You just commit.

## What to do

From `workspace/`, create a file to commit:

```
echo "# Paired to the store" > PAIRING.md
```

Stage it and commit with a normal AIDA-format message (you did this in
exercise 11 — `[AI:tool] type(scope): description (REQ-ID)`):

```
git add PAIRING.md
git commit -m "[AI:claude] docs(pairing): note the store trailer (FR-1)"
```

Now look at what the hook added:

```
git log -1
```

The message body carries an `Aida-Store:` line you never typed. Confirm
the pairing from AIDA's side too:

```
aida store status
```

It prints the code HEAD, the store SHA your commit was paired with, the
current store HEAD, and whether they're aligned.

## Verify

`aida-tutor verify` — passes once a commit touching `PAIRING.md` exists
*and* its message carries an `Aida-Store:` trailer. If it fails, you
committed with `--no-verify` (which skips the hook) or the hook is
missing — `aida store install-hook` reinstalls it.

## Tip

`aida init` installs the hook automatically; `aida store install-hook`
is the retrofit path for a project that predates the feature. The hook
deliberately skips merge, squash, and amend commits — those inherit
their trailer from the source commit.

## What's next

Exercise 32 — turn requirements into a review brief with `aida review
prompt`.
