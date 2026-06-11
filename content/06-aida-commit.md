## Goal

Stage and commit your trace-commented file using AIDA's commit format.
The commit-msg hook should green-check it.

## Why

AIDA's convention for commit messages is:

```
[AI:tool] type(scope): description (REQ-ID)
```

Each piece does work:

- **`[AI:tool]`** — flags AI-assisted commits. The validator warns if
  staged files have `trace:` comments but the message lacks `[AI:...]`.
- **`type`** — conventional-commits style: `feat`, `fix`, `docs`,
  `chore`, `refactor`, `test`, `style`, `perf`, `build`, `ci`, `revert`.
- **`(scope)`** — optional area / module touched.
- **`description`** — short, present-tense.
- **`(REQ-ID)`** — the requirement(s) this commit advances. The
  validator checks for `(FR-1)` form on `feat` and `fix` commits;
  comma lists `(FR-1, BUG-2)` and ranges `(BUG-25..30)` also work.

Result: `git log --oneline` reads as a backlog walk. Every commit
points at a req; every req's history is reconstructible from
`git log -- :path-with-trace`.

## What to do

From `workspace/`:

```bash
git add <your-traced-file>
git commit -m "[AI:claude] feat(<scope>): <description> (<FR-id>)"
```

Replace `<your-traced-file>`, `<scope>`, `<description>`, `<FR-id>`
with sensible values.

The commit-msg hook will run automatically (it was installed by
`aida init`) and print `✓ Commit message format valid` on success.

## Tip

If the hook complains, read the error — it tells you precisely which
piece is missing. Most common stumble: forgetting the `(REQ-ID)` at
the end.

## Verify

`aida-tutor verify` parses your last commit message and checks both
the conventional shape and the REQ-ID parens.
