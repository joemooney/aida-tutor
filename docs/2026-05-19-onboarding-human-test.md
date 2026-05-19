# Onboarding slice — moderated human test (EPIC-5 / TASK-2)

**Purpose.** Validate EPIC-5 **AC-1**: does the "one wow" land for a real
first-time AIDA user? This is the gate the advisor handoff set — *no
investment beyond the slice (the Act II restructure) until this passes.*
Building and merging the slice (PR #11) proved it runs; only a real newcomer
can prove the wow lands.

**The wow being tested.** After the learner captures one intent, has an agent
implement it with a `// trace:` comment, and commits — `aida show <SPEC>`
reveals the spec⇄commit⇄file round-trip, and a fresh agent session answers
about the work from the graph. The realization to listen for: *"the project
itself now holds a queryable memory, and my coding agent shares it — the
conversation is disposable, the project's memory isn't."*

## Recruit (1–2 testers)

- Uses a coding agent (Claude Code) day to day.
- Has **never used AIDA**.
- Not an AIDA contributor; hasn't seen this tutorial.
- A working developer. One tester is the minimum; two is better.

## Setup (before the tester arrives)

- A machine with `aida` installed, Claude Code available, the `aida-tutor`
  repo cloned.
- Confirm `aida-tutor onboard` launches cleanly.
- Arrange to record the session (screen + audio, with consent) or be ready
  to take close real-time notes.

## Run it (~15–20 min)

Hand the tester **only this**:

> "Run `aida-tutor onboard` and follow it. Think out loud — tell me what
> you're seeing and what you expect to happen."

Then **moderate, don't teach**:

- Do **not** explain AIDA, the steps, or what's coming. Do not lead.
- Let them work the slice unaided. Step in only if genuinely stuck — and
  **write down where and why** (that is itself a finding).
- Watch their face and listen closely at the reveal (step `05`) and the
  cold-agent step (step `06`).

Record per step: time taken (total target 10–15 min); where they light up
vs glaze / reread / sigh; their **exact words** at steps 05–06; any point
they would plausibly have quit.

## The verdict — AC-1 pass / fail

Ask, **after** they finish (never before, never leading):

> "In your own words — what did that just show you?"

- **PASS** — unprompted, they articulate the shared-memory realization:
  *"the project itself knows this now,"* *"the next session already has the
  context,"* *"I didn't have to re-explain it to the agent,"* or equivalent.
- **FAIL** — they land only on the floor: *"nice, it tracked my
  requirement,"* *"ok, a requirements tracker"* — or show no reaction at the
  reveal.
- **FAIL (slice, not wow)** — they glaze or would quit before reaching steps
  05–06. The wow concept may be sound but the slice is too long/slow —
  tighten the losing step and re-test.

Then one more, unleading: *"Would you keep using AIDA after that? Why or why
not?"*

## Decision

- **Pass** (both / majority of testers) → EPIC-5 is validated. The Act II
  restructure becomes considerable — subject to the handoff's other ripeness
  condition (AIDA's orchestrator surface quiet for 2+ weeks).
- **Fail** → **do not** start Act II. Iterate the slice — the framing line,
  or the specific step that lost them — and re-test. A failed test here is
  the cheap save the whole thin-slice strategy was designed to buy.

---

*Protocol authored 2026-05-19 by the aida-tutor advisor. Source:
`docs/2026-05-18-advisor-handoff.md` (decisions #3, #4), EPIC-5 AC-1 / AC-6,
`docs/plans/2026-05-18-first-contact-onboarding-slice.md`.*
