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

From `workspace/`, run `aida list`. You should see five rows — one per
type: VIS-1, PRIN-1, ADR-1, FR-1, BUG-1. Each type has its own counter
starting at 1, so the prefix tells you the type and the number is the
nth-of-that-type. The display order is most-recently-modified first, so
you'll likely see BUG at top and VIS at bottom.

## Tip

When the list gets long, lean on filters. `aida list --status approved
--priority high` is the natural "what should I do next?" query.

## Optional: stricter verification

`aida list` leaves nothing on disk, so this exercise — like the other
read-only ones (8, 13, 15, 16, 17) — passes once the prerequisite state
exists, whether or not you actually ran the command.

If you'd like the tutor to hold you to it, install the optional
invocation-logging wrapper:

```
aida-tutor wrapper
```

It drops a tiny `aida` shim into `workspace/.aida-tutor-bin/`; put that
directory first on your `PATH` and every `aida` call is logged. With the
wrapper active, these exercises verify you actually ran the command. It
is off by default — `aida-tutor wrapper --uninstall` (or any `aida-tutor
reset`) removes it.

## Verify

`aida-tutor verify` — checks that all five req types now exist (and, if
the wrapper is installed, that you ran `aida list`).
