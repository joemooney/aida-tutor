## Goal

Write an implementation plan under `docs/plans/` and lint it with
`aida plan verify`.

These last three exercises cover AIDA's *maintenance and integration*
surface — the admin commands you reach for less often than the daily
capture loop, but want to know exist: plan linting, store auditing, and
the MCP bridge that connects AIDA to an AI agent.

## Why

A plan is a promise: "here's what I'll change, here's how I'll know it
worked." AIDA archives plans as markdown under `docs/plans/` — one file
per planned piece of work, committed to the repo next to the code.

But a plan rots. You reference `src/foo.rs:42`, the file grows, and
line 42 is now something else. You list a file that later gets renamed.
You skip the section that says how to verify the work. Each of those is
a small lie the next reader trusts.

`aida plan verify <file>` is the lint that catches them. It checks:

- **Drifted `path:line` refs** — a `file.rs:42` whose line moved.
- **Missing files** — a path the plan names that no longer exists.
- **Absent required sections** — a plan with no `## Critical Files`,
  `## Verification`, or `## Followups` section.

It exits non-zero on any error, so it runs as a pre-commit hook on
`docs/plans/`. A plan that passes `verify` is a plan you can trust.

## What to do

From `workspace/`, create the plans directory:

```
mkdir -p docs/plans
```

Write `docs/plans/2026-05-18-readme-note.md`. The three sections the
linter *requires* are `## Critical Files`, `## Verification`, and
`## Followups`; `## Summary`, `## Approach`, `## Files`, and
`## Related` round out the template. Point every file reference at
`README.md` — it exists in your workspace, so the file check passes:

```
# Note the store pairing in the README

## Summary
Add a line to README.md describing the AIDA store.

## Approach
Append one sentence under the project title.

## Critical Files
- `README.md` — the file the plan edits

## Files
| Action | File |
|--------|------|
| Modify | `README.md` |

## Verification
Run `aida plan verify` on this file and confirm a PASS verdict.

## Followups
- None.

## Related
- FR-1 — the feature captured in exercise 05
```

Then lint it:

```
aida plan verify docs/plans/2026-05-18-readme-note.md
```

It prints an OK / WARN / ERROR line per check and a final verdict.
Recommended sections only WARN — the verdict is PASS as long as there
are zero ERRORs.

## Verify

`aida-tutor verify` — passes once a file under `docs/plans/` carries
the three required sections (`## Critical Files`, `## Verification`,
`## Followups`). A plan missing one of them fails `aida plan verify`,
so it fails here too.

## Tip

`aida plan verify --fix` rewrites drifted `path:line` refs in place to
the corrected line numbers. And `aida plan helpers <SPEC>` does the
inverse of linting — it harvests the `// trace:` comments of sibling
specs so a new plan reuses existing helpers instead of reinventing
them.

## What's next

Exercise 34 — audit the requirement store's health with `aida db info`
and `aida db check`.
