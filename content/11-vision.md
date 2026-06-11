## Goal

Capture your project's **vision** — what it is, in one sentence.

## Why

Visions are the highest layer of an AIDA project's canonical narrative.
They sit above features and bugs. The point of a vision isn't to plan
implementation — it's to declare intent so every later "should we add X?"
has a yardstick.

Visions are typically `--status approved` (not `draft`) because they're
declarations, not proposals. Once you and your collaborators agree on
the vision, it changes rarely.

## What to do

From inside `workspace/`, add a vision requirement. Use the `vision` type.

The title should be a single sentence stating what the project is. The
description (multi-paragraph is fine) explains the user-facing target —
what someone will be able to do with this thing once it's built.

Set status to `approved`. Priority isn't critical here — `medium` or `high`
both fit.

## Tip

If your title contains backticks or single quotes, wrap it in double
quotes (or escape carefully). The shell will eat unquoted special
characters and AIDA will warn you about a "suspicious title".

## Verify

Run `aida-tutor verify`. The verifier looks for any `VIS-*` requirement
in your store, with status approved.
