# Plan: First-contact onboarding — the 15-minute shared-memory wow

Date: 2026-05-18
Specs: EPIC-5, STORY-31, STORY-32, STORY-33, STORY-34, TASK-2
Status: Draft
Complexity: ~350 prod LOC (mostly content markdown), ~120 test LOC, ~6 commits, risk medium

<!--
  AIDA plan template. The risk here is not the code — it is whether the wow
  lands. That risk is retired by the TASK-2 moderated human test, not by this
  plan. Build cheap, test on humans, then decide. trace:EPIC-5
-->

## Approach

aida-tutor is being repurposed from a 17-exercise rustlings-style grind into
**onboarding for new AIDA users**. Per the 2026-05-18 advisor handoff, this is a
**thin slice, not the full restructure**: a single new first-contact path that
delivers **one genuine "oh, I get it" moment in 10–15 minutes**, built only on
AIDA's *stable* surface (spec graph, capture, trace comments, `aida show/list/
search`, the code↔spec round-trip, commits). The volatile agent-collaboration
layer (queue, roles, sessions, `--auto-complete`, autonomy modes) is **out of
scope** — deferred to a future "Act II" restructure.

The 17 existing exercises stay as-is as "Act I." This work *prepends* a new
path; it does not rewrite anything. The new path is reached via a new
`aida-tutor onboard` command and one discoverability line in `cmd_welcome()`.

The **one wow** is the round-trip / shared-queryable-memory moment: the learner
captures one real intent as a spec, hands it to a coding agent that implements
it *and* writes the `// trace:` comment, commits — then `aida show <SPEC>`
reveals the spec⇄commit⇄file linkage that assembled itself. The wow is **not**
"I tracked a requirement" (the floor — the "I could do this in 20 lines of
bash" reaction is the failure mode). The wow is the **cold-start contrast**: a
fresh agent session with zero conversational context answers a question about
the work correctly because it reads the project's memory over MCP.

Build it cheap in days, then gate every further investment on a **moderated
human test** (TASK-2). One observed session beats weeks of guessing.

### Diagram

```
  aida-tutor onboard
        │
  step 0  why-you're-here  ──────────────────────────────┐
  step 1  aida init scratch project                      │ ~10 min
  step 2  capture intent      →  FR-1 (spec)              │ SETUP
  step 3  agent implements    →  code + // trace:FR-1     │
  step 4  commit              →  [AI:claude] feat (FR-1)  │
        │                                                 │
  step 5  aida show FR-1      →  spec ⇄ commit ⇄ file ────┤ THE REVEAL
  step 6  cold agent, /clear  →  answers from the graph ──┤ THE WOW
  step 7  forward signpost    →  "Act II, when it settles"┘ ~5 min

  verifiable on-disk:  spec file · trace comment · commit subject · status
  experiential only:   aida show output · the cold-agent answer  (human test)
```

## Decisions

- **Decision: thin slice, not the audit's full restructure.** **Rationale**:
  inherited DECIDED from the advisor handoff. The orchestrator surface churned
  ~15× in one day; teaching it now guarantees drift. The stable surface broke
  0/17 exercises in 8 days. Build on stable; defer Act II until that surface is
  quiet 2+ weeks *and* the human test confirms the wow.

- **Decision: confirm the candidate "one wow" — round-trip / shared queryable
  memory — and sharpen it to the cold-start contrast.** **Rationale**: it is
  the only candidate where the learner *builds the artifact that then wows
  them* inside the 15 minutes (a `search`-finds-old-decision wow needs
  pre-seeded history that is not the learner's own). The sharpening: the wow is
  not `aida show` alone (a skeptic answers "git log --grep does that") — it is
  the fresh, context-free agent answering from the graph. That dramatizes
  AIDA's own stated pain ("agents start every session cold"). Default framing
  line, to be calibrated by the human test: *"I closed the session — the next
  agent picked up exactly where I left off, because the project remembered,
  not the chat."*

- **Decision: the slice runs inside Claude Code; the agent is real, not
  simulated.** **Rationale**: AIDA's audience *is* Claude Code users; assuming
  a coding agent is safe. A told wow ("imagine an agent…") is weaker than a
  shown one, and decision #2 of the handoff forbids a manufactured wow.

- **Decision: do NOT build the invocation-recording verifier shim now.**
  **Rationale**: that shim is restructure-scoped infrastructure (it exists to
  close the e08–e17 read-only gap across the whole tutor). The thin slice
  sidesteps the gap instead: its *verifiable* checkpoints are all
  state-changing (spec created, trace comment present, commit links spec,
  status flip) — inspectable by the existing `src/verify.rs` model. The
  read-only payoff steps (`aida show`, the cold-agent query) are validated by
  the human test, not by Rust. Keeps the build in *days*.

- **Decision: new `src/onboarding.rs` module, not new `exercises/eNN_*.rs`
  files.** **Rationale**: the slice is one linear guided track with a shape the
  `Exercise` trait does not fit (it seeds a scratch project, has a cold-agent
  step). Reuse `verify.rs` helpers and the `VerifyResult` enum; do not
  shoehorn into the 17-exercise registry.

- **Decision: minimal discoverability only — one `cmd_welcome()` line + one
  README line.** **Rationale**: the full framing re-anchor (README,
  `cmd_welcome()`, CLAUDE.md narrative) is restructure scope. The slice only
  needs to be *findable*, not to re-spine the product.

- **Decision: scratch project is a ~20–30 line Python CLI.** **Rationale**:
  lowest setup friction, readable on sight by most developers; the slice is
  about AIDA, not the language. The hello-world task is one agent-turn-sized
  feature add (e.g. a `--upper` flag on a `greet` command).

## Files (in build-order)

### `content/onboarding/scratch-template/` — the stub project (new)

- A ~20–30 line Python `greet` CLI + a minimal README. Seeded into `workspace/`
  by the onboard command; `workspace/` stays gitignored and reset-safe.

### `content/onboarding/00-why.md` … `07-signpost.md` — step content (new)

- Eight markdown files, one per step in the diagram. Mirrors the existing
  `content/NN-slug.md` convention. Owns STORY-32; steps 5–7 owned by STORY-34.

### `src/onboarding.rs` — the slice engine (new)

- `struct OnboardingSlice`: ordered steps, each with a content slug + an
  optional on-disk verifier.
- `fn seed_workspace`: copy `scratch-template/` into `workspace/`.
- `fn verify_step`: dispatch to the `verify.rs` inspectors for the four
  state-changing checkpoints. Owns STORY-31 (seed/entry) + STORY-33 (verifiers).

### `src/verify.rs` — inspection helpers (extend, reuse first)

- Reuse existing on-disk inspectors. Add only if missing:
  `trace_comment_present(path, spec)` and `commit_subject_references(spec)`.

### `src/main.rs` — wire the command (extend)

- Add `Cmd::Onboard` variant to the `Subcommand` enum (near `main.rs:24`).
- Add `cmd_onboard()` dispatch (alongside `cmd_show`/`cmd_verify`).
- `cmd_welcome()` (`main.rs:524`): one line — "New to AIDA? Run `aida-tutor
  onboard` for the 15-minute tour."

## Critical Files

- `content/onboarding/` (step markdown + `scratch-template/`)
- `src/onboarding.rs`
- `src/verify.rs`
- `src/main.rs`

## Reusable helpers (do not reimplement)

- `src/verify.rs` — the existing read-only on-disk inspectors (`.aida-store`
  YAML reads, `git log` greps, trace greps). The slice's verifiers are new
  *callers* of these, not new inspection machinery.
- `VerifyResult` enum (`src/exercise.rs`) — reuse for the slice's step results;
  do not invent a parallel result type.
- The `demo()` subprocess-driving pattern from STORY-19 (`aida-tutor demo`) —
  if the slice gets a non-interactive walkthrough, copy that pattern rather
  than re-deriving subprocess plumbing.
- `aida init` already scaffolds `.mcp.json` — the cold-agent step relies on
  that wiring; do not hand-roll MCP setup.

## Risks + gotchas

1. **Risk**: the wow does not land — testers hit the "floor" reaction ("nice,
   a requirements tracker"). **Mitigation**: AC-1 names that reaction as the
   explicit *failure* signal; TASK-2's moderated human test is a hard gate
   before any further investment. This is the single highest risk and it is
   retired by observation, not by building more.

2. **Risk**: the cold-agent step (step 6) depends on the learner's Claude Code
   being MCP-wired. **Mitigation**: `aida init` scaffolds `.mcp.json`; step 1
   surfaces/checks it. If MCP is unavailable, the step degrades gracefully to
   a human-run `aida show` (the round-trip still shows; the agent-reads-it
   half becomes told, not shown — weaker but not broken).

3. **Risk**: scope creep toward the full Act II restructure. **Mitigation**:
   EPIC-5 non-goals are explicit; the deferred items live in Followups below;
   the advisor pushes back if a queue item drifts toward them.

4. **Risk**: 15 minutes is optimistic — the agent-implementation step (step 3)
   is variable. **Mitigation**: the hello-world task is scoped to one agent
   turn; the human test measures *actual* wall-clock and feeds AC-2.

5. **Risk**: slice drift as AIDA evolves. **Mitigation**: stable-surface-only
   is AC-3 (0/17 broke in 8 days of churn); once PR #1's CI matrix lands, the
   slice rides the same `aida-tutor demo` gate.

## Tests (named, not "add tests")

- `onboard_verify_spec_created` — the captured intent lands as a YAML object
  in `.aida-store`. Happy path.
- `onboard_verify_trace_comment_present` — the scratch code carries
  `// trace:<SPEC>` after step 3.
- `onboard_verify_commit_links_spec` — the commit subject references the SPEC.
- `onboard_verify_status_flipped` — the spec leaves `approved`.
- `onboard_seed_is_reset_safe` — re-seeding `workspace/` does not touch
  `.aida-tutor-progress.toml`. Invariant.
- `onboard_demo_walks_green` — non-interactive end-to-end walkthrough (extends
  the STORY-19 `demo` pattern), if adopted.

## Verification

```bash
TMP=$(mktemp -d) && cd "$TMP"
git init -q
# 1. launch the slice
aida-tutor onboard            # seeds workspace/ with the scratch project
cd workspace
# 2. drive the round-trip by hand (what a learner does)
aida init
aida add --type functional --status approved --title "greet --upper flag"
# ...agent implements + writes // trace:FR-1, then:
git commit -am "[AI:claude] feat(cli): add --upper flag (FR-1)"
# 3. assert the verifiable checkpoints
grep -rq 'trace:FR-1' .                         && echo OK trace
git log --oneline | grep -q '(FR-1)'            && echo OK commit-link
aida show FR-1 | grep -qiE 'commit|file'        && echo OK round-trip renders
# 4. the wow itself is NOT asserted here — it is observed in the TASK-2
#    moderated human test. CI proves the linkage exists; humans prove it wows.
```

## Followups

Out of scope now — do NOT pull these into EPIC-5. Revisit per the handoff's
"when the full restructure becomes ripe" gate.

- The full "Act II" agent-collaboration restructure (queue/roles/sessions/
  orchestrator arc).
- The `aida` invocation-recording verifier shim that closes the e08–e17
  read-only gap.
- Fix the 4 drifted Act-I exercises the audit flagged (ex.5, 7, 8, 14, 16).
- The ex.14 `done` vs `completed` concept fix (belongs at the Act I/II seam).
- Full framing re-anchor of README / `cmd_welcome()` / CLAUDE.md narrative.

## Related

- Builds on: `docs/plans/2026-05-18-aida-tutor-audit.md` (the gap audit),
  `docs/2026-05-18-advisor-handoff.md` (the re-scoping decision — wins on
  conflict with the audit).
- Gated by: TASK-2 (moderated human test) — the investment gate.
- See also: `/home/joe/ai/aida/OVERVIEW.md` ("the TUI is the product",
  Trojan-horse positioning), `/home/joe/ai/aida/CLAUDE.md` (stable vs volatile
  surface).
