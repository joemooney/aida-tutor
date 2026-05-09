## Goal

Capture a **functional requirement** (FR) — a behavior the system must do.

## Why

Functional requirements are the bread-and-butter of project work. They
describe what the system does from a user's perspective. Each FR
eventually corresponds to (a) a chunk of code and (b) trace comments
linking that code back to this FR.

FRs are status-tracked: `approved` → `in-progress` → `completed`. The
status moves as work happens. AIDA's `/aida-pickup` slash command picks
the highest-priority `approved` FR off the queue when a developer or
agent says "what should I do next?".

## What to do

From `workspace/`, add a requirement of type `functional`. (Type is
`functional`; prefix is `FR`.)

Title: a behavior the system must do, phrased as a verb-y action
("Parse JSON input", "Persist user preferences"). Description: enough
detail that someone could start implementing — what's in scope, what
isn't, edge cases worth flagging.

Status: `approved`. Priority: `high` (we're going to implement it
shortly).

## Tip

If your description is more than three paragraphs, the FR is probably
too big — split it into multiple smaller ones. AIDA tracks parent/child
relationships if you want to keep them grouped.

## Verify

`aida-tutor verify` — looks for any `FR-*` requirement with status approved.
