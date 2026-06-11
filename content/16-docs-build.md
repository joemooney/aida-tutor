## Goal

Project the canonical layer (vision, principle, decision) into a
human-readable markdown tree.

## Why

`aida docs build` walks the requirement graph, picks out the docs-layer
entities (Vision, Principle, Decision/ADR, Constraint, Glossary Term),
and writes them out as a layered markdown tree under `docs/aida/`. The
layout follows arc42 conventions:

```
docs/aida/
├── README.md            # index — what's where
├── 00-constitution.md   # principles
├── 01-vision.md         # vision(s)
├── 02-constraints.md    # constraints
├── 05-decisions/        # one ADR per file
│   ├── README.md
│   └── ADR-1-<slug>.md
├── 07-quality.md        # non-functional requirements
└── 10-glossary.md       # terms
```

The graph is the source. The docs tree is a derived view — re-run
`aida docs build` whenever you add/change canonical reqs.

## When to run it

- After adding new vision/principle/decision/term/constraint reqs
- Before a release (so the docs reflect the release state)
- When onboarding someone new (the README + layered files read as a
  tour of the project's intent)

Not after every commit. Weekly is plenty.

## What to do

From `workspace/`, run `aida docs build`.

After it finishes, browse the result:

```bash
ls -R docs/aida/
cat docs/aida/README.md
cat docs/aida/01-vision.md
cat docs/aida/00-constitution.md
```

## Tip

The `--dry-run` flag shows what *would* change without writing. Useful
in CI to gate on "the docs are in sync with the graph".

## Verify

`aida-tutor verify` checks that `docs/aida/README.md` and at least one
layer file exist.
