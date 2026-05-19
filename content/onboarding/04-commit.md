## Step 4 — commit, and name the spec

Commit the change with the spec id in the message — that trailing `(FR-1)`
is the load-bearing part:

```
git add greet.py
git commit -m "[AI:claude] feat(greet): add --upper flag (FR-1)"
```

The `(FR-1)` makes this commit part of FR-1's history — not in a tracker
you have to keep in sync, but in the commit itself, where it can never
drift away from the code.

Three things now name FR-1: the spec, the trace comment in `greet.py`, and
this commit. You wrote each of them once, as an ordinary act. You never
drew a line between them — and yet they're connected.
