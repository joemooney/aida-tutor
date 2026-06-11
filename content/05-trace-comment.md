## Goal

Write a code file with a `trace:` comment linking it back to FR-1.

## Why

Trace comments are the most distinctive AIDA pattern. They're short
machine-readable pointers from code → spec, written in source comments:

```rust
/// Convert ts in `from` zone to `to` zone. trace:FR-1 | ai:claude
pub fn convert_at(...) -> ... { ... }
```

The format is `trace:<SPEC-ID> | ai:<tool>[:<confidence>]`:

- `trace:FR-1` — points at requirement FR-1
- `ai:claude` — this code was written with Claude's help (high confidence)
- `ai:claude:med` — Claude wrote it but the design was mostly yours
- `ai:claude:low` — minor AI assist, you wrote most of it
- (omit `ai:` entirely if no AI was involved)

Tooling reads these comments to:

- **`aida search`** — find code by spec id
- **The commit-msg validator** — warn if a commit touches files with
  `trace:FR-X` comments but doesn't reference FR-X in the message

The trick is the comments scale: every `pub` item in a feature can carry
a `trace:` and the project graph knows where each piece of code lives.

## What to do

In `workspace/`, create a source file (`src/something.rs` is a fine
choice — the directory will be created). Put any code you like in it,
but include at least one `trace:<your-FR-id> | ai:claude` comment.

A doc comment (`///` or `//!`) and a regular comment (`//`) both work.

## Tip

You can have multiple traces in one file — both module-level and per-
function — and `aida search` will pick them all up.

## Verify

`aida-tutor verify` greps `workspace/` for `trace:FR-...` comments.
