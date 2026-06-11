## Goal

Close the loop on FR-1: mark it finished with `aida done`.

## Why

You've come full circle: captured FR-1, built it, linked the code to the
spec, and watched `aida show` prove the link. The last step is telling
AIDA you're finished.

The newcomer-friendly way to do that is one command:

```bash
aida done FR-1
```

`aida done` flips the status to `completed` — so `aida list --status
approved` no longer shows it, the recent-activity log records the close,
and `aida show` reflects it. It's the no-jargon shortcut for what you
could also spell out as `aida edit FR-1 --status completed`; same effect,
fewer words to remember.

That's the whole core loop: **capture → build → link → done.** Everything
past here is optional.

## What to do

From `workspace/`:

```bash
aida done FR-1
```

Then, if you like, leave a note for the audit trail — context that doesn't
belong in the description, like a follow-up or a decision you made while
working:

```bash
aida comment add FR-1 "v0 stub committed. Real wiring per ADR-1 still TODO."
```

The comment is optional, but cheap, and it's what makes the trail useful
six months from now. `aida show FR-1 --comments` renders it back.

## Tip

`aida done` is one of a small family of newcomer shortcuts (alongside the
positional `aida add "..."` you used in exercise 02). Once you're fluent,
`aida edit --status ...` gives you the full status vocabulary — see the
"Going further" exercises.

## Verify

`aida-tutor verify` — checks any `FR-*` is `completed` (what `aida done`
sets).
