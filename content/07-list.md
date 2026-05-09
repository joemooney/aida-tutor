## Goal

Read your project's whole backlog with one command.

## Why

`aida list` is the everyday "where am I?" view. It shows every
requirement, type, status, priority, and title. By default META
requirements (AI prompt customization seeded at init) are hidden so
your real work isn't drowning in plumbing — pass `--include-meta` if
you want to see them.

Most filters work as flags:

- `aida list --status approved`
- `aida list --type functional`
- `aida list --priority high`
- `aida list --tags auth,login`

Combine them.

## What to do

From `workspace/`, run `aida list`. You should see five rows: VIS-1,
PRIN-1, ADR-1, FR-1, BUG-1 (numbers may differ slightly depending on
ordering inside your block).

## Tip

When the list gets long, lean on filters. `aida list --status approved
--priority high` is the natural "what should I do next?" query.

## Verify

`aida-tutor verify` — checks that all five req types now exist.
