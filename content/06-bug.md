## Goal

Capture a **bug** — a known issue with the project.

## Why

AIDA bugs can be filed BEFORE the code that contains the bug exists.
Capturing reasoning is a feature, not a misuse. If you're thinking
through a feature and notice "wait, what about leap years?", file a
bug right then. It becomes the parking spot for that thought, gets a
stable id, and surfaces in `aida search` later when someone hits it.

Once an FR is `completed`, any open bugs against it should be visible
in the project pulse — `aida status` will show them in Recent activity.

## What to do

From `workspace/`, add a requirement of type `bug`. Title: the symptom
in one line ("Leap-year date arithmetic off by one"). Description: the
reproduction (or the reasoning that led you to suspect it), the
expected vs actual behavior, and any links to the FR or code area
this affects.

Status: `approved` (we agree it's a real bug we want to fix). Priority:
your call — `high` for blocker-class issues, `medium` or `low` for
papercuts.

## Tip

Bugs that lose context are bugs that get re-filed. Always include
*why this is a bug, not a feature gap* — it's the difference between
"the system doesn't do X" (feature request) and "the system does X
wrong" (bug).

## Verify

`aida-tutor verify` — looks for any `BUG-*` requirement.
