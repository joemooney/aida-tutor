## Goal

Capture a **principle** — a non-negotiable rule the project will follow.

## Why

Principles answer questions before they're asked. "Should we use UTC or
the user's local time?" "Should errors return Result or panic?" Principles
are the place to answer once. New contributors (and AI agents) read them
and immediately know how the project behaves.

In `aida docs build`, principles get rendered into the `Constitution`
layer — they're the project's foundational rules.

## What to do

From `workspace/`, add a requirement of type `principle`. Title: a clear
imperative ("Default to UTC", "All errors are values, not exceptions").
Description: the rationale + a small example.

Status: `approved`. Priority: `high` (principles are load-bearing).

## Tip

Principles aren't aspirations. They're things you'll enforce. If you'd
write "we should try to..." it probably belongs in a feature, not a
principle.

## Verify

`aida-tutor verify` — looks for any `PRIN-*` requirement.
