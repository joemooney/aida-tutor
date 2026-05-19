## Goal

Generate a **review brief** — a markdown checklist built from
requirements' acceptance criteria — with `aida review prompt`.

## Why

A reviewer's job is to check the change against what was *asked for*.
But the ask lives in the requirement store, and the reviewer is looking
at a diff. Bridging the two from memory is how "looks fine to me" slips
past a missed acceptance criterion.

`aida review prompt` closes that gap. Give it a set of specs and it
emits a markdown brief: each requirement's title, its acceptance /
verify section, and a decision rubric — approve, or request changes
*tied to the spec_id* so the contributor can act on it by id, not by
paraphrase.

Hand that brief to a reviewer — a teammate, or an AI agent in a fresh
session — and the review is anchored to the spec instead of vibes.

## What to do

From `workspace/`, build a brief for the feature you captured in
exercise 05. Confirm its id first:

```
aida list --type functional
```

It's `FR-1`. Generate the brief and write it to a file with `--write`
(otherwise it just prints to stdout and is gone):

```
aida review prompt --specs FR-1 --write review-brief.md
```

Open `review-brief.md`. You'll see a `# Review Prompt` header, a
`## What to verify` section with a `### FR-1` entry, and a `## Decide`
rubric. `--specs` takes a comma-separated list — `--specs FR-1,BUG-1`
briefs several requirements at once.

## Verify

`aida-tutor verify` — passes once `review-brief.md` exists and looks
like a generated brief: a `# Review Prompt` header and a `## What to
verify` section naming at least one spec.

## Tip

`aida review prompt` also reads specs straight from a pull request:
`aida review prompt --pr 1` parses the `(REQ-ID)` trailers off every
commit in the PR's range and briefs all of them together. That form
needs the GitHub or GitLab CLI (`gh` / `glab`) installed to resolve the
PR — handy once you're reviewing real PRs, out of scope for this
offline workspace. `--write` works there too.

## What's next

That's the last exercise — and the whole tour. You've walked AIDA end
to end: `init`, capture, trace comments, AIDA-format commits, the
distributed orphan store, roles and the producer/consumer queue, the
requirement graph, scoped sessions with worktrees, and now commit
pairing and the review workflow. Run `aida-tutor progress` to see
32/32. Go capture your own project's first requirement.
