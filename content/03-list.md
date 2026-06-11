## Goal

See your captured work in the everyday "where am I?" view.

## Why

`aida list` is the command you'll run most. It shows every requirement —
type, status, priority, and title — one row each. By default META
requirements (AI prompt customization seeded at init) are hidden so your
real work isn't drowning in plumbing — pass `--include-meta` to see them.

Most filters work as flags, and compose:

- `aida list --status approved`
- `aida list --type functional`
- `aida list --priority high`
- `aida list approved` (the bare positional form of `--status approved`)

## What to do

From `workspace/`, run:

```bash
aida list
```

You should see **FR-1** — the feature you captured in exercise 02 — with
its type, status, and priority. Right now it's the only row of real work
(you'll add more requirement types later, in "Going further"). Each type
has its own counter, so the prefix tells you the type and the number is
the nth-of-that-type.

## Tip

When the list gets long, lean on filters. `aida list --status approved
--priority high` is the natural "what should I do next?" query.

## Optional: stricter verification

`aida list` leaves nothing on disk, so this exercise — like the other
read-only ones (show, search, status, push) — passes once the
prerequisite state exists, whether or not you actually ran the command.

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

`aida-tutor verify` — checks FR-1 exists (and, if the wrapper is
installed, that you ran `aida list`).
