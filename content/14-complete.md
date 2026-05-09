## Goal

Close the loop on FR-1: mark it completed and add a comment recording
how it landed.

## Why

When work finishes, two things should happen:

1. **Status flips to `completed`** — so `aida list --status approved`
   no longer shows it, recent activity records the close, and the
   docs build / status output reflect it.
2. **A comment captures context that doesn't belong in the description**
   — commit sha, "deferred ambiguous-time handling to BUG-X",
   "spoke to teammate Y about edge cases", links to PR. The
   description tells *what the req is*; comments tell *what happened
   while working it*.

Both are cheap. Together they make the audit trail dense and useful
six months from now.

## What to do

From `workspace/`, run two commands:

```bash
aida edit <FR-id> --status completed
aida comment add <FR-id> "<your note about how it landed>"
```

Sample notes that age well:

- `"v0 stub committed (sha abc1234). Real wiring per ADR-1 still TODO."`
- `"Implemented in src/something.rs. Tested manually; see BUG-2 for follow-up."`
- `"Done. Spoke to <person> about scope; we're punting <thing> to v2."`

## Tip

Comments accept multi-line text via shell quoting. They also support
markdown-ish formatting that `aida show --comments` will render
faithfully.

## Verify

`aida-tutor verify` — checks any `FR-*` is `completed` AND has at
least one comment.
