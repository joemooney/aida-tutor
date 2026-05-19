## Step 5 — the reveal

The feature is built, so close the loop on it:

```
aida edit FR-1 --status completed
```

Now ask the project what it knows:

```
aida show FR-1
```

Read what comes back. The spec text and its status, yes — but also a
**Git linkage** section: the commit that implemented FR-1, and `greet.py`,
the file carrying the trace comment.

You never built that linkage. You captured a spec, wrote one comment,
wrote one commit message — three ordinary acts — and AIDA assembled them
into a queryable record of *how this feature came to be*.

That is the round trip: intent to code to commit, and back again. Ask
`aida show FR-1` six months from now and the answer is still there.
