# CLAUDE.md

Guidance for Claude Code working in this repository.

@.claude/AIDA.md

## Project overview

**aida-tutor** is a hands-on tutorial for [AIDA](../aida/), in the spirit of
[rustlings](https://github.com/rust-lang/rustlings). 17 exercises that walk a
learner from `aida init` through the full capture → trace → commit → close
loop, with an on-disk verifier per exercise.

Status: **v0 complete and end-to-end verified** as of 2026-05-09. All 17
exercises pass when worked in order.

## Architecture

- `src/main.rs` — CLI: `list`, `show`, `verify`, `hint`, `reset`, `progress`.
- `src/exercise.rs` — `Exercise` trait + `VerifyResult` enum.
- `src/exercises/eNN_slug.rs` — one verifier + hint per exercise.
- `src/verify.rs` — pure inspection helpers (read-only).
- `src/progress.rs` — completion record persisted at `.aida-tutor-progress.toml`.
- `content/NN-slug.md` — exercise descriptions in markdown.
- `workspace/` — where the learner does the exercises (gitignored).

The tutor dogfoods AIDA: this repo has its own `.aida-store/` with the vision,
principles, decisions, and exercise stories all filed as proper requirements.
Run `aida list` here to see them.

## Working on the tutor

Daily commands:

```bash
cargo build --release                                   # build
cargo test                                              # smoke (no tests yet)
./target/release/aida-tutor reset --yes && \
  ./target/release/aida-tutor show                      # rehearse the flow
aida list                                               # browse the tutor's own reqs
```

When changing an exercise:

1. Edit `content/NN-slug.md` and `src/exercises/eNN_slug.rs`.
2. Run a fresh smoke pass: reset workspace, do the exercise by hand, run
   `verify` after every step.
3. Update `aida edit STORY-N --status completed` if the story isn't already.

When adding a new exercise:

1. Pick the next id (`NN`) and a kebab-case slug.
2. Create `content/NN-slug.md`.
3. Create `src/exercises/eNN_slug.rs` impl-ing `Exercise`.
4. Register in `src/exercises/mod.rs`.
5. File a STORY in the AIDA store (`aida add --type story ...`).
6. Smoke-test by walking the new exercise end-to-end.

## Open work / handoff

The tutor's AIDA store has STORY-1..17 (one per exercise) — all are
`approved` or `completed` depending on whether the verifier exists.
EPIC-1 (CLI v0) and EPIC-2 (Tutorial Exercises) are the umbrellas.

**Next-up backlog ideas** — file as new stories when picking them up:

- **`aida-tutor watch`** — auto-re-run verify on `workspace/` changes
  (similar to rustlings' watch). Inotify on Linux, polling fallback.
- **`aida-tutor demo`** — non-interactive: run all exercises in order
  via subprocess, verify each, exit non-zero on any pending. Useful in CI.
- **Hint depth levels** — current hint is one paragraph. A `--more`
  flag could surface a multi-step nudge before showing the final
  "here's the literal command" escape valve.
- **More exercises** — relationships (`aida rel add`), the queue
  (`aida queue add`), roles (`aida role enter`), `aida history`,
  `/aida-pickup` from inside Claude Code.
- **Verifier rigor** — exercises 7, 8, 13, 15, 16, 17 currently pass
  on prerequisite state alone (we can't tell whether the user actually
  ran a read-only command). A workspace-level `aida` wrapper that
  records invocations could close that gap.

## Important conventions

- `workspace/` is the learner's playground. Verifiers READ from it,
  never write. This is PRIN-1 in the tutor's own store.
- Progress lives at the repo root (`.aida-tutor-progress.toml`),
  NOT inside `workspace/`, so `reset` doesn't wipe progress.
- All exercise content is markdown in `content/` — Rust knows only
  the slug + verifier. Editing content is a non-Rust task per ADR-2.
- The tutor uses `aida` as a subprocess (ADR-1) to test the real
  CLI surface, not the library API.

## Linked AIDA store (the real one — for changes that affect AIDA itself)

This tutor's existence surfaces shortcomings in AIDA. When you find
one (e.g. "wow it would be nice if `aida` had a JSON output mode for
verifiers to consume"), file it in `~/ai/aida/` (the AIDA repo), not
here. The tutor's own AIDA store is for tutor-internal work only.
