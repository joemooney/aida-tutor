# aida-tutor — Advisor Handoff Brief

**From:** the AIDA project advisor session · **Date:** 2026-05-18
**To:** the aida-tutor advisor (you)

This is a context transfer. You are picking up the advisor (`role:dialog`) seat for
**aida-tutor**. This brief gives you the strategic aim, what is already decided (do not
relitigate), what is your call, and your first actions. It is checked in so future
aida-tutor sessions can re-read it.

---

## The mission

aida-tutor is being **repurposed**: from a rustlings-style exercise grind into
**onboarding for new AIDA users**. The job is to walk a newcomer from zero to a genuine
*"oh — I get it"* moment, fast enough that they don't wander off to a different tool first.

There is real time pressure. The project owner has people watching who *"may go in
different directions if I cannot show material utility."* Onboarding is on the critical
path to AIDA adoption — but see the "first impressions are sticky" warning below before
you rush.

---

## Required reading (in order)

1. **`docs/plans/2026-05-18-aida-tutor-audit.md`** — the gap audit of the current 17
   exercises. TL;DR: the exercises are **not broken** (0/17), but they teach only the
   requirements-capture *floor* and miss the entire agent-collaboration layer (19 missing
   concepts). The audit recommended a full restructure — that recommendation has since
   been **re-scoped** (see "What's decided" below).
2. **`/home/joe/ai/aida/CLAUDE.md`** and **`/home/joe/ai/aida/OVERVIEW.md`** — AIDA's own
   positioning. Read the *"Trojan-horse"* framing and the *"the TUI is the product"*
   section specifically. AIDA's defensible niche is the **agent-collaboration layer**
   (queue, orchestrator, roles, trace graph, MCP). The requirements-capture surface is
   the *floor* — it invites the *"I could do this in 20 lines of bash"* reaction. A
   tutorial that stops at the floor mis-sells the product.

---

## What's DECIDED — do not relitigate

The audit said "full restructure." After a strategy discussion that was re-scoped. The
decisions you inherit:

1. **Thin slice now — NOT the full restructure.** AIDA has two surfaces with opposite
   volatility:
   - **STABLE** — spec graph, capture, trace comments, `aida show` / `list` / `search`,
     the code↔spec round-trip, commits. Evidence: 0 of 17 exercises broke in 8 days of
     heavy AIDA evolution. **Safe to build onboarding on today.**
   - **VOLATILE** — the orchestrator, `--auto-complete`, `--zen` / `--no-human`, queue
     mechanics, roles, sessions. ~15 bugs and changes in a *single day*. Teaching this in
     depth now guarantees the tutorial drifts before it ships.

   Build the onboarding on the **stable** surface. The full "Act II" restructure that
   deeply teaches the agent-collaboration layer is **deferred** (see "When the full
   restructure becomes ripe").

2. **One genuine wow, ~15 minutes — not a 10-second gasp.** AIDA's value is
   *medium-burn*: it "surfaces through use," per AIDA's own CLAUDE.md. Do **not**
   manufacture a fake instant wow. Design for the 10–15-minute *"oh, I see it"*.

3. **First impressions are sticky — a thin onboarding that over-promises and underwhelms
   is WORSE than none.** The watchers you're trying to keep will calibrate on it. The
   *one wow* must be genuinely good. Quality of the one wow beats quantity of exercises.

4. **Test on real humans — don't guess.** Build the slice, sit 1–2 real people down,
   watch where they light up vs glaze over. One observed session beats weeks of
   speculating "would they wow." (This mirrors AIDA's own SPIKE-7/SPIKE-8 evidence-first
   discipline.)

5. **Build in days, not weeks.** The time pressure is real; the thin slice is the
   response to it. If scope creeps toward the full restructure, push back.

---

## The candidate "one wow" — pressure-test it, it is not gospel

Strong candidate: the **round-trip / queryable-memory** wow.

The newcomer captures intent (a spec) → an agent writes code carrying a
`// trace:FR-1` comment → `aida show FR-1` reveals the project answering questions about
*itself* that the user would otherwise have to remember. The wow is **not** "I tracked a
requirement" (that is the floor — the exact reaction the audit warns against). It is:

> **"My project now has a queryable memory — and my coding agent shares it."**

That is the MCP story, and it sits entirely on the stable surface.

**This is your call to confirm, sharpen, or replace.** You own the precise framing and
the moment it lands.

---

## What's YOUR call (your latitude as aida-tutor advisor)

- The exact **hello-world task** — what tiny *real* coding thing the newcomer builds.
- The precise **"one wow" framing** and the exact moment it lands.
- The **step breakdown** of the thin slice (how many steps, in what order).
- How the slice **points forward** at the depth without deep-teaching the volatile
  orchestrator layer (a closing "…and there's a whole orchestration layer underneath —
  next session" beats teaching it badly now).
- Whether to also close the **e08–e17 verifier gap** the audit flagged, as part of this
  effort or as a separate follow-up.
- The decomposition into queue items and the order of work.

---

## First actions

1. Read the required reading above.
2. Confirm, sharpen, or replace the "one wow."
3. File an **EPIC in aida-tutor's own `.aida-store`** (`aida add --type epic …` run from
   this repo — *not* AIDA's store). The thin first-contact onboarding slice. Its **first
   acceptance criterion** should be the "one wow," defined concretely enough to test.
4. Write a rewrite plan in `docs/plans/` using AIDA's plan template.
5. Decompose into queue items; drive them.
6. When the slice is ready, surface back to the project owner for a **human test** — that
   is the gate before any further investment.

---

## When the full restructure becomes ripe

Revisit the audit's full-restructure recommendation when **both** are true:

- (a) AIDA's agent-collaboration surface has been **quiet for 2+ weeks** — no orchestrator
  / `--zen` / `--no-human` churn; and
- (b) the thin slice's **human test confirms the wow lands**.

Until then: the 17 existing exercises stay as **Act I**, as-is. Fix their cosmetic drift
(the audit lists ~4–5 drifted exercises) only if it is cheap and convenient — it is not
the priority. The priority is the thin first-contact slice.

---

*Handoff authored 2026-05-18 by the AIDA project advisor. If this brief and the audit
disagree, this brief wins — it post-dates the audit and carries the re-scoping decision.*
