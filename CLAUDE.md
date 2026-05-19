# CLAUDE.md

Guidance for Claude Code working in this repository.

@.claude/AIDA.md

## Project overview

**aida-tutor** is a hands-on tutorial for [AIDA](../aida/), in the spirit of
[rustlings](https://github.com/rust-lang/rustlings). 32 exercises that walk a
learner from `aida init` through the full capture → trace → commit → close
loop, into distributed storage, the roles + queue workflow, the requirement
graph, scoped sessions with git worktrees, and the code-review + commit-
pairing workflow, with an on-disk verifier per exercise.

Status: **v0 complete and end-to-end verified** as of 2026-05-09 (exercises
01–17). EPIC-4 cluster 1 adds the distributed-storage trio (18–20: orphan
branch, store sync, cache rebuild); cluster 3 adds the roles + producer/
consumer queue arc (21–24: role enter, queue add `--for`, pickup, done);
cluster 2 adds the relationships pair (25–26: `add --parent`, `rel add`);
cluster 4 adds the sessions + worktrees quartet (27–30: session start,
work-in-worktree, leases/show, session end); cluster 5 adds the code-review
+ commit-pairing pair (31–32: `Aida-Store:` trailer, `review prompt`).
All 32 exercises pass when worked in order (`aida-tutor demo`).

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

**Shipped CLI polish** — `aida-tutor watch` (STORY-18, auto-re-run verify
on `workspace/` changes), `aida-tutor demo` (STORY-19, non-interactive CI
walkthrough), three-depth hints (STORY-20): `hint` → `hint --more`
(multi-step nudge) → `hint --solution` (the literal command), with
`--solution` use recorded as `completed-with-solution` in progress, and
`aida-tutor wrapper` (STORY-22, opt-in invocation-logging shim — see
Verifier rigor below).

**Next-up backlog ideas** — file as new stories when picking them up:

- **More exercises** — `aida history`, sessions + cross-project queue
  routing. Relationships shipped as 25–26 (STORY-26); roles + the
  producer/consumer queue as 21–24 (STORY-27); sessions + worktrees as
  27–30 (STORY-28); code review + commit pairing as 31–32 (STORY-29);
  plans + maintenance + MCP as 33–35 (STORY-30). The `aida review
  prompt --pr` form is taught in 32's content but not verified — the
  PR-driven path needs `gh`/`glab` + a real forge remote, out of reach
  for the offline workspace.
- **Verifier rigor** — `aida-tutor wrapper` (STORY-22) installs an
  optional workspace-local `aida` shim that logs every invocation to
  `.aida-tutor-invocations.log`; once it's first on `PATH` the read-only
  exercises 7, 8, 13, 15, 16 and 17 verify the command actually ran
  rather than passing on prerequisite state alone. Off by default,
  wiped by `reset`. Remaining gap: exercises 29 (`session leases`/`show`)
  and 34 (`aida db info`/`db check` audit — leaves no on-disk trace)
  still pass on prerequisite state; `verify_invocation`'s single-token
  subcommand match doesn't yet cover the two-level `aida session ...`
  form.
- **Counter-scope migration** — STORY-30's original sketch included an
  `aida config numbering` exercise, dropped because `aida config` is
  "not supported for git backend" (the default mode) as of AIDA 0.7.0.
  Worth an exercise once that command works on the git-canonical store.

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

## Sync ritual with AIDA (added 2026-05-10)

aida-tutor runs **behind** AIDA — every major capability shipped in
`~/ai/aida/` should land here as one or more exercises. Until
cross-project queue routing lands (AIDA's STORY for that is captured
under EPIC-22 — Cross-project AIDA primitives), the sync is manual:

**When a new AIDA EPIC ships:**

1. Read its acceptance criteria via `aida show EPIC-N` (in `~/ai/aida/`)
2. Decide what the learner needs to *do* to demonstrate the capability
3. File a STORY in this tutor's own AIDA store (`cd ~/ai/aida-tutor`,
   `aida add --type story --parent EPIC-2 --title "exercise eXX:
   <topic>"`) — EPIC-2 here is the tutor's "v0.2 post-EPIC-9 coverage"
   parent, see `aida list --type epic` in the tutor store
4. Implement the verifier under `src/exercises/eXX_<slug>.rs` and the
   content under `content/XX-<slug>.md`
5. Wire it in `src/exercises/mod.rs`
6. CI gates the green build on every exercise verifying against the
   latest AIDA release (see `Cargo.toml` `aida-min-version` constant
   and `.github/workflows/exercises.yml` once those land — both are
   tracked in the v0.2 EPIC).

**Coverage gap signal:** if `aida list --type epic --status completed`
in `~/ai/aida/` shows EPICs that don't appear under EPIC-2's children
in this repo, you're behind. The release script in `~/ai/aida/scripts/
release.sh` may eventually warn about this; until then, eyeball it.

**Cross-project pointer (for future tooling):** the project registry
at `~/.aida/projects.toml` lists both repos. When AIDA's cross-project
queue lands, an aida-side release will be able to `aida queue add
TUTOR-COVERAGE-EPIC-N --to-project aida-tutor --for implementer`
automatically. Today this is manual.
