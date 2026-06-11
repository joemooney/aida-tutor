## Goal

Find requirements by keyword.

## Why

`aida search "<query>"` runs FTS5 (full-text search) against every
requirement's title AND description. It's fast (sub-millisecond) and
indexed automatically as you add reqs.

Use it when you remember "something about auth" but not the spec id.
Or when picking up a thread from a previous session — search for the
keyword you're working on and see every related req.

## What to do

From `workspace/`, run `aida search "<keyword>"` — pick a word that's
in at least one of your req titles or descriptions.

For example, if your principle was about UTC and your bug was about
timezone handling, `aida search "timezone"` would hit both.

## Tip

`aida grep "<regex>"` is the regex flavor — same FTS index, but you
can do case-insensitive (`-i`), or restrict to a specific field
(`-f description`). For most everyday work, plain `aida search` is
enough.

## Verify

Read-only command. `aida-tutor verify` passes once prerequisite state
exists.
