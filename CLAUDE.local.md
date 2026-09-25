# CLAUDE.local.md — Personal notes for Claude Code

This file is **per-machine** and **gitignored**. It loads at the start of every
Claude Code session in this project, just like `CLAUDE.md`, but it never leaves
your machine. Use it for project-specific rules and personal-habit reminders
that belong to you, not the team.

If you're new to this file: the pattern comes from the Claude Code
community. After every PR review, dump the feedback here. Over time it
becomes a personalized rule file capturing exactly the kind of mistakes
you make most often. Reviewer nits drop noticeably within a couple
weeks of consistent use.

---

## Project review feedback (private)

Rules you've learned from reviewer comments on YOUR PRs in this repo.
Add new lines as you see the feedback. Phrase each as an imperative rule.

<!-- Examples — replace with your own:
- New SQS consumers need a DLQ and alarms in the same PR
- Use `Optional<T>` over null returns
- Tests for new endpoints must include the auth-failure case
- Prefer named tuples over plain dicts for return types with 3+ fields
-->

## Personal habits to correct

Things YOU keep doing that you want Claude to remind you about (or just
do for you). These are about your interaction style and shortcuts, not
project conventions.

<!-- Examples — replace with your own:
- Stop using `console.log`; use the project logger instead
- Always update the OpenAPI spec when adding endpoints
- Run `bun run typecheck` before claiming done (Claude does this for you)
-->

---

## Tips for maintaining this file

- **Keep sections clearly separated.** Project feedback ≠ personal habits.
- **Prune every few weeks.** Things that have become muscle memory can go.
  The file should capture what you're still learning, not what you already
  do automatically.
- **For PROJECT-WIDE rules** (everyone on the team needs them) — those
  belong in `CLAUDE.md` (which is committed), not here.
- **For GENERAL AIDA patterns** that apply across all your AIDA projects —
  those belong in `~/.claude/projects/<slug>/memory/feedback_*.md`, not
  here.
- **`aida findings add`** is the right verb for a pattern observation that
  hasn't earned its rule status yet. Let recurrence promote it.

The `/aida-learn` skill (scaffolded by `aida init`) walks through the
routing decision when you don't know which substrate a rule belongs in.
