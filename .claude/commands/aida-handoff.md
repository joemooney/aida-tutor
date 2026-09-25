---
description: Run /aida-handoff.
argument-hint: [SPEC-ID]
allowed-tools: Bash, Read, Grep, Edit, Write
---
<!-- AIDA Generated: v2.0.0 | checksum:46bfb39d | DO NOT EDIT DIRECTLY -->
<!-- To customize: copy this file and modify the copy -->


# AIDA Handoff

Codex prompt arguments: `$ARGUMENTS`

Follow the workflow in `.claude/skills/aida-handoff/SKILL.md`:

1. Capture conversation-only residue using the `/aida-capture` discipline.
2. Run `aida session handoff --check` and inspect drains, leases, briefs, and open PR/review state.
3. Write a durable handoff note or brief for the next session. If `$ARGUMENTS` names a spec, use that as the focus.
4. End with one decision line: `Recommendation: START FRESH` or `Recommendation: COMPACT`, listing every pin when compaction is required.