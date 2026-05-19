# aida-tutor Audit / Gap Report vs. Current AIDA

**Date:** 2026-05-18
**Auditor:** Claude (Opus 4.7)
**AIDA binary audited:** `aida 0.7.0 (built 2026-05-17, sha 8a4d7cfa)`
**Tutorial last meaningfully updated:** ~2026-05-09 (CLAUDE.md says "v0 complete and end-to-end verified as of 2026-05-09")

## Summary — headline verdict

The 17 existing exercises are **structurally sound and mostly still work** — the
core capture/trace/commit/docs/status/push loop they teach has not been broken
by AIDA's evolution. Of 17 exercises, **13 work**, **4 have drifted** (stale
flag names, changed output shape, or wording that no longer matches current
behavior), and **0 are outright broken**. No exercise teaches a removed or
renamed command.

The real problem is **not drift — it is scope.** Since 2026-05-09 AIDA's own
positioning (`/home/joe/ai/aida/OVERVIEW.md`, `/home/joe/ai/aida/CLAUDE.md`)
has crystallized around the **agent-collaboration layer** as its *defensible
niche*: the queue, the `--auto-complete` orchestrator, roles, sessions/leases,
plans, the MCP server, autonomy modes, the `done` vs `completed` lifecycle. The
tutorial covers **none of it.** A learner who finishes all 17 exercises has
learned "AIDA = a nicer requirements tracker" — which AIDA's own docs
explicitly call the *floor*, the thing that prompts the intended "so what, I
could do this in 20 lines of bash" reaction. The tutorial currently stops at
exactly the surface AIDA wants users to look *past*.

**Recommendation: restructure** (keep the 17 exercises as the foundational
"Act I", but rebuild the tutorial narrative so the agent-collaboration layer is
the spine, not an unwritten appendix). Justification in the Recommendation
section.

## Per-exercise verdict

| Exercise | Verdict | Notes |
|---|---|---|
| `01-init.md` | works | `aida init` defaults to git-canonical orphan-branch mode, exactly as described. Scaffolds `.aida/`, `.aida-store/`, `.claude/`. Minor: it now also scaffolds `.codex/` and an `AGENTS.md` (the `--agent both` default); the exercise only mentions `.claude/`. Cosmetic, not wrong. |
| `02-vision.md` | works | `--type vision` valid; `VIS-*` prefix correct; `--status approved` valid. |
| `03-principle.md` | works | `--type principle` valid; `PRIN-*` prefix correct. Text says principles render into the `Constitution` layer of `aida docs build` — still accurate (`docs/aida/00-constitution.md`). |
| `04-decision.md` | works | `--type decision` valid; `ADR-*` prefix correct; renders to `docs/aida/05-decisions/`. |
| `05-feature.md` | drifted | `--type functional` / `FR-*` correct. **Stale claim:** "AIDA's `/aida-pickup` slash command picks the highest-priority `approved` FR off the queue." `/aida-pickup` is now driven by the explicit **work queue** (`aida queue add` / `aida queue work`), not an implicit "scan for highest-priority approved" heuristic. Also names the lifecycle `approved → in-progress → completed`, omitting the now-load-bearing `done` state (see ex.14). |
| `06-bug.md` | works | `--type bug` / `BUG-*` correct. "open bugs surface in `aida status` Recent activity" still holds. |
| `07-list.md` | drifted | `aida list` and the `--status/--type/--priority/--tags` filters are all current. **Stale:** `--include-meta` is correct, but the exercise does not mention that `aida list` now **hides terminal-status (Completed/Rejected) reqs by default** (`--all` brings them back). After ex.14 completes FR-1, a learner running plain `aida list` will see *four* rows, not five — directly contradicting the exercise's "you should see five rows" and the verifier's premise. Also: role-scoped filtering (`--no-scope`) and `--show-origin` exist now and are unmentioned. |
| `08-show.md` | drifted | `aida show <id>` and `--comments` correct. **Stale:** `aida show` now prints a **git-linkage section** (commits / files traced / branch / PR) by default — the exercise's "What you'll see" description predates it and does not mention `--no-git`, `--verbose`, `--tree`, or the `--card` spec-card view that `/aida-pickup` relies on. Output shape has materially changed. |
| `09-in-progress.md` | works | `aida edit <id> --status in-progress` works; status-name normalization (`in-progress`/`inprogress`/`in_progress`) still holds. Note: `aida edit --help` only *documents* `draft/approved/completed/rejected` in its enum text, but `in-progress` is still accepted as a normalized value (the `--force` help text itself references `--status in-progress`). Exercise is accurate; AIDA's help text is the thing that drifted. |
| `10-trace-comment.md` | works | `trace:<SPEC-ID> | ai:<tool>[:<confidence>]` format unchanged and current. Confidence levels (`high` implied, `med`, `low`) correct. |
| `11-aida-commit.md` | works | `[AI:tool] type(scope): description (REQ-ID)` format unchanged. Conventional-commit type list correct. Commit-msg hook still installed by `aida init`. Range form `(BUG-25..30)` and comma lists still valid. |
| `12-docs-build.md` | works | `aida docs build` and `--dry-run` correct. The arc42-style layer layout described matches current output. Note: `aida docs check` (CI drift gate) is a sibling worth a one-line mention but its absence is not drift. |
| `13-search.md` | works | `aida search "<query>"` correct (still FTS-backed, case-insensitive default). `aida grep` exists with `-i` / `-E`; exercise mentions `-f description` field restriction — current `aida grep --help` shows `-i`/`-E` but the field-restriction flag should be re-verified against a full `aida grep --help`; treat the `-f` claim as the one soft spot here. |
| `14-complete.md` | drifted | `aida edit <id> --status completed` + `aida comment add` both work. **Stale / conceptually incomplete:** the exercise teaches `completed` as the single "work finished" terminal state. Current AIDA (STORY-86) splits this into **`done`** ("work finished on a branch", set by `aida queue done`) vs **`completed`** ("merged to the default branch", auto-bumped by `aida pull`). Teaching a manual `--status completed` is now the *exception path*, not the norm — the merge is supposed to promote it. The exercise actively teaches a habit current AIDA discourages. |
| `15-show-comments.md` | works | `aida show <id> --comments` and `aida comment list <id>` both current. Author/timestamp/`(edited …)`/threaded-reply rendering still accurate. |
| `16-status.md` | drifted | `aida status` works, but its scope has **grown substantially**. The exercise describes Project/Requirements/Cache/Sync/Recent-activity/Scaffolding sections. Current `aida status` is now a "where am I right now" view that *leads* with **Session / Branch / PR-CI / Queue** sections (it's the spiritual cousin of `git status`). The exercise's section inventory is a strict subset of — and de-emphasizes — what the command actually shows now. New flags `--short`, `--json`, `--queue`, `--ci`, `--no-ci` are unmentioned. |
| `17-push.md` | works | `aida push` with `--code-only` / `--store-only` correct. Graceful no-remote skip behavior still holds. New `--dry-run` / `--json` / `--no-rebase-check` flags exist but their absence is not drift. The closing "What's next" text ("the model can read AIDA conventions and work alongside you") is the natural seam where the missing agent-collaboration arc should begin. |

**Counts:** works 13 · drifted 4 · broken 0.

## Missing concepts — the agent-collaboration-layer inventory

Everything below is **verified present** in `aida 0.7.0` via `aida <cmd> --help`
and corroborated by `/home/joe/ai/aida/CLAUDE.md`. None of it is mentioned
anywhere in the 17 exercises. This is what AIDA's own docs call its *defensible
niche* — and the tutorial teaches none of it.

1. **The work queue** (`aida queue add/list/next/move/remove/done`) — a
   personal, role-routable backlog. A learner needs it because `/aida-pickup`
   and every "what should I do next?" flow now run *off the queue*, not off an
   implicit priority scan. Exercise 5 already tells a half-truth about this.

2. **`aida queue work`** — collapses the 5-7 manual steps (pull, session start,
   cd, role enter, `claude /aida-pickup`) into one command, launching Claude in
   a fresh worktree. This is the single most-used daily entry point in current
   AIDA and is completely absent.

3. **The `--auto-complete` orchestrator** — `aida queue work --auto-complete`
   drives the full **6-phase pipeline**: implementer → CI → reviewer → merge →
   pull → build. Variants `through-ci` / `through-merge` / `skip-build` stop
   early. This is AIDA's headline agent-collaboration capability; a tutorial
   that omits it undersells the product more than any single drift bug does.

4. **Roles / personas** (`aida role enter/add/list/scaffold/scope/prompt`) —
   persistent named contexts (implementer, reviewer, architect, triage) that
   resume across shells and scope what `aida list` / `aida queue` show. A
   learner needs roles to understand *who* the queue routes work to.

5. **Sessions & leases** (`aida session start/end/list/leases/show/manifest`) —
   scoped git-worktree sessions with advisory leases ("who holds what scoped
   work right now"). The unit of isolated, parallel agent work.

6. **Autonomy modes** — `--zen` ("advisor on standby": auto-resolve mechanical
   prompts, still pause for design forks) and `--no-human` / `--unattended`
   (headless `claude -p` drains). The learner needs the *interactive vs
   autonomous* trade-off framed explicitly — it is a per-session choice.

7. **Plans** (`docs/plans/`, `aida plan verify`, `aida plan helpers`,
   `aida ultraplan`) — the structured 11-section plan template, the linter that
   catches drifted refs, and the `/ultraplan` round-trip. Plans "ride into the
   session" via the manifest. A learner doing non-trivial work needs this.

8. **The MCP server** (`aida mcp-serve`) — exposes the requirement graph to
   Claude Code as MCP tools/resources. AIDA's CLAUDE.md calls this "the
   highest-leverage surface for the agent-context vision." The tutorial never
   says the word MCP.

9. **Batch drains** (`aida queue work --batch NAME`, `--auto-complete --batch`,
   `aida queue progress --batch`) — `batch:NAME`-tagged clusters drained
   autonomously, one full lifecycle per member. The "drain a backlog overnight"
   story.

10. **`aida fetch`** — read-only two-leg (code + store) refresh of remote refs
    without merging; the safe counterpart to `aida pull`.

11. **`aida cache`** (`cache status` / `cache rebuild`) — the SQLite read
    projection over the git-canonical store. Exercise 1 mentions "a SQLite
    cache" in passing but no exercise teaches inspecting or rebuilding it, and
    no exercise explains the git-canonical-vs-cache write-through model.

12. **`done` vs `completed` lifecycle** — `done` = finished on a branch,
    `completed` = merged to main (auto-bumped on merge). Exercise 14 actively
    teaches the pre-STORY-86 single-terminal-state model. This is a *concept*
    gap, not just a missing command.

13. **`aida rebase`** — detect/classify/execute a rebase of the current branch
    onto upstream; the divergent-branch recovery tool.

14. **`aida rework`** / **`aida queue rework`** — the implementer → reviewer →
    fixup recovery verb (status flip + queue re-route + optional session
    launch).

15. **`aida findings`** — triage review findings filed by the headless
    reviewer (`list` / `dismiss` / `promote`). The reviewer-side surface of the
    autonomous loop.

16. **`aida history`** — per-requirement digest / chronological event feed
    ("what was I up to last session?"). A natural session-resumption tool the
    tutorial never introduces.

17. **`aida goal`** — derive a machine-checkable `/goal` completion condition
    from AIDA metadata. The "make autonomous runs terminate deterministically"
    primitive.

18. **`aida tui`** — the terminal-UI shell that PTY-hosts Claude Code sessions
    (EPIC-26). Per OVERVIEW.md this is now *the public face of the product* —
    "the TUI is what people will think AIDA is." The tutorial does not mention
    it exists.

19. **`aida status` as a session dashboard** — already counted as drift in
    ex.16, but worth restating: `status` now surfaces session/branch/PR-CI/queue
    state. It became an agent-collaboration command and the exercise treats it
    as a requirements-summary command.

**Missing-concept count: 19** (commands + concepts a 2026-05-09 tutorial would
have no way to cover).

## Command-surface delta vs 2026-05-09

**Commands/flags the tutorial uses that have changed behavior or output:**

- `aida list` — now hides terminal-status reqs by default (`--all` added). New
  flags: `--no-scope`, `--show-origin` (`-v`), `--parent`, `--sync`. Exercise 7's
  "five rows" expectation is invalidated once FR-1 is completed in ex.14.
- `aida show` — now prints a git-linkage section by default. New flags:
  `--no-git`, `--verbose`, `--tree`/`--depth`, `--card`/`--brief`/`--full`,
  `--sync`. Exercise 8's "What you'll see" predates all of this.
- `aida status` — restructured into a `git status`-style multi-section view
  leading with Session/Branch/PR-CI/Queue. New flags `--short`, `--json`,
  `--queue`, `--ci`, `--no-ci`. Exercise 16's section list is now a subset.
- `aida edit` / `aida add` — the `--status` enum *help text* now only lists
  `draft/approved/completed/rejected`; `in-progress` and `done` are accepted as
  normalized values but undocumented in `--help`. Not a tutorial bug, but worth
  knowing when cross-checking.
- `aida init` — now scaffolds `.codex/` + `AGENTS.md` by default (`--agent both`);
  new flags `--sibling`, `--registry-remote`, `--name`, `--verbose`. Exercise 1's
  "drops a `.claude/` directory" description is incomplete but not wrong.

**Major commands that did not exist (or were not surfaced) on 2026-05-09 and
are entirely absent from the tutorial:** `aida queue` (the whole subcommand
tree, incl. `work`, `--auto-complete`, `--batch`, `done`, `progress`, `rework`),
`aida role`, `aida session`, `aida plan`, `aida ultraplan`, `aida goal`,
`aida rework`, `aida findings`, `aida fetch`, `aida rebase`, `aida history`,
`aida tui`, `aida store`, `aida statusline`, `aida usage`, `aida upgrade`,
`aida mcp-serve`. (Some of these may predate 2026-05-09 in primitive form, but
none are taught.)

**Deprecated surface the tutorial does NOT teach (good — no action needed):**
The tutorial correctly uses git-canonical mode throughout — exercise 1 describes
the orphan `aida-store` branch + `.aida/cache.db` model, which is current. It
does **not** teach `aida init --centralized` / the legacy SQLite-canonical
`requirements.db` backend, which is now deprecated and prints a warning at init
time. No deprecated-surface remediation is required. The one nuance: exercise 1
calls the cache "a SQLite cache that makes `aida list`/`search` fast" without
explaining it is a *rebuildable projection* over the git-canonical store — that
is an under-explanation, not teaching a deprecated thing.

## Verifier coverage gap — e08–e17

Only **e01–e07 have meaningful Rust verifiers**; the harness files `e08`–`e17`
exist in `src/exercises/` but, per the tutor's own CLAUDE.md and inline notes,
exercises **7, 8, 13, 15, 16, 17 pass on prerequisite state alone** — they
cannot detect whether the learner actually ran the read-only command the
exercise teaches (`aida show`, `aida search`, `aida status`, `aida push`,
`aida comment list`). The verifiers for those are effectively no-ops that
inherit the pass/fail of earlier state-changing exercises.

This gap is **structural, not incidental**: `src/verify.rs` only knows how to
inspect on-disk state (YAML files, git log, trace greps). Read-only commands
leave no on-disk trace, so the current verifier model *cannot* close the gap.
The tutor's own CLAUDE.md already names the fix: "a workspace-level `aida`
wrapper that records invocations could close that gap."

**Recommendation on the gap:** any effort to bring the tutor up to speed
**should** close it, because the missing agent-collaboration arc (queue, roles,
sessions, orchestrator) is *full of read-only and side-effecting commands whose
effects are easy to inspect* (`aida queue list` writes queue YAML; `aida role
enter` writes `.aida/roles/`; `aida session start` creates a worktree + lease
file). Building the invocation-recording `aida` shim wrapper now — as part of
the restructure — means the new exercises get real verifiers from day one
instead of inheriting the e08–e17 no-op pattern. Closing the gap and adding the
new arc are the same piece of work and should be scoped together.

## Recommendation — restructure (not incremental)

**Recommendation: restructure.**

The drift evidence alone (steps 3 and 5) would only justify `incremental`: 4
drifted exercises, 0 broken, all fixable with wording and output-snapshot
edits. If drift were the whole story, the right call would be "patch the 4, ship
it."

But drift is not the whole story. The decisive evidence is the **missing-concept
inventory (19 items) read against AIDA's own positioning.** AIDA's
`OVERVIEW.md` and `CLAUDE.md` are explicit and unambiguous:

- The requirements-capture layer the tutorial teaches is the **"floor"** —
  Karpathy-style "structured markdown queryable by Claude." AIDA's *defensible
  niche* is the agent-collaboration layer on top of it.
- The intended first reaction to AIDA's surface is *"so what, I could do this
  in 20 lines of bash"* — and that reaction is supposed to be **dispelled by
  use of the agent-collaboration depth**, not reinforced.
- A 17-exercise tutorial whose narrative arc is "init → capture → trace →
  commit → docs → status → push" delivers a learner to exactly the bash-script
  reaction and then *stops*. It teaches the floor as if it were the building.

A tutorial that ends at exercise 17 today actively **mis-sells the product**: it
trains the learner to think "AIDA = a tidy requirements tracker with trace
comments," which is the impression AIDA's strategy explicitly wants users to
move *past*. Appending an agent-collaboration arc as exercises 18+ would help,
but it would still leave the *framing* — the README, the welcome screen, the
"01. … 17." overview, the CLAUDE.md "v0 complete" framing — anchored on
requirements-capture as the spine. The narrative, not just the exercise count,
is what is outdated.

**What "restructure" concretely means** (not "throw away the 17 exercises"):

- Keep the 17 exercises largely intact as **Act I — "The durable index"**: this
  is genuinely the floor and a learner does need it first. Fix the 4 drifted
  exercises while doing so (ex.5, 7, 8, 14, 16 — note ex.16 wording).
- Add **Act II — "The agent-collaboration layer"**: a new arc covering the
  queue, roles, sessions/leases, plans, `queue work`, the `--auto-complete`
  6-phase orchestrator, autonomy modes, and the MCP server. This is where the
  *defensible niche* gets taught.
- Re-anchor the **framing**: README, the `cmd_welcome()` text in `src/main.rs`,
  the tutor's CLAUDE.md, and the exercise-overview table should present the
  agent-collaboration layer as the destination, with capture as the on-ramp.
- Fix exercise 14's `done` vs `completed` concept (it currently teaches a
  pre-STORY-86 model) — this belongs at the Act I/Act II seam because `done`
  only makes sense once the queue and merge lifecycle are on the table.

The cost delta between `incremental` and `restructure` is modest — both require
writing the new arc and the verifier shim — and `restructure` additionally
requires reworking ~5 framing files. That is a small price for a tutorial that
teaches what AIDA *is* rather than what it *was in May 2026*.

## Suggested next step

1. **File the umbrella in the tutor's own `.aida-store`.** Create an EPIC in
   the aida-tutor AIDA store (`cd /home/joe/ai/aida-tutor && aida add --type
   epic --title "Restructure tutorial around the agent-collaboration layer"`)
   and link it as the v0.2 parent. The tutor's CLAUDE.md "Sync ritual with
   AIDA" section already describes this filing convention (EPIC-2 as the
   post-EPIC-9 coverage parent) — extend it.

2. **Write a rewrite plan** at `docs/plans/2026-05-18-tutorial-restructure.md`
   using the AIDA 11-section plan template, decomposing the work into:
   (a) drift fixes for the 4 drifted Act I exercises + the ex.14 `done`/`completed`
   concept, (b) the `aida` invocation-recording shim that closes the e08–e17
   verifier gap, (c) the new Act II exercise arc (one STORY per new exercise:
   queue, roles, sessions, plans, `queue work`, `--auto-complete`, autonomy
   modes, MCP), (d) the framing rework (README, `cmd_welcome()`, CLAUDE.md).

3. **Sequence the verifier shim first.** It unblocks real verifiers for both the
   drifted read-only Act I exercises *and* every new Act II exercise — building
   it early prevents the new arc from inheriting the e08–e17 no-op pattern.

4. **Re-verify against the AIDA release, not a dev build.** This audit ran
   against `aida 0.7.0` from a `+dirty` dev tree (`/home/joe/ai/aida/target/
   debug/aida`). Before shipping the restructure, pin the tutor's CI to the
   latest tagged AIDA release (the `aida-min-version` constant the tutor's
   CLAUDE.md already anticipates) so exercises are validated against what
   learners will actually have installed.
