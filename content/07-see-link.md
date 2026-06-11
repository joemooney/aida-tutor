## Goal

Run `aida show FR-1` one more time and watch the code↔spec link appear.

## Why

This is the moment the whole tool is built around.

Back in exercise 04 you ran `aida show FR-1` and the **Git linkage**
section was empty — nothing in the repo pointed at FR-1 yet. Since then
you:

- wrote a `// trace:FR-1` comment in your code (exercise 05), and
- committed it with `(FR-1)` at the end of the message (exercise 06).

Run the *same* `aida show FR-1` now and the linkage section has filled
in. AIDA reconstructed it from your git history and your trace comment —
you didn't tell it twice, you just did the normal work and referenced the
id once in each place.

## What to do

From `workspace/`:

```bash
aida show FR-1
```

Look near the bottom for a block like:

```
Git linkage:
  Branch     <branch>
  Commits (1)
    1cd0070 [AI:claude] feat(parser): scaffold JSON parser (FR-1)
  Files traced (1)
    src/parser.rs — parse_json
```

There it is: from the spec you can find the commit and the exact file
that implements it. From the code (the `trace:` comment) you can find the
spec. The link is two-way, and it cost you nothing beyond naming the id.

## Tip

This is why the `(REQ-ID)` trailer and the `trace:` comment matter. They
look like bookkeeping in the moment; six months later they're how anyone —
human or agent — answers "why does this code exist?" and "is this spec
actually built?" without guessing.

## Verify

`aida-tutor verify` runs `aida show FR-1` and confirms the **Git linkage**
section now lists your commit and traced file.
