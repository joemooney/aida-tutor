## Goal

Capture an **architectural decision** (ADR) — a specific technical
choice plus the reasoning behind it.

## Why

Code shows what you chose. ADRs show **why**. Every project accumulates
forks-in-the-road: which database, which auth library, which deployment
target. Writing those down — with the trade-offs you considered — saves
the next person from re-litigating the choice.

The convention is named after Michael Nygard's "Architectural Decision
Record" article. AIDA gives them their own type so they're searchable
and projected separately by `aida docs build`.

## What to do

From `workspace/`, add a requirement of type `decision`. (Note: type is
`decision`, prefix on the id is `ADR`. AIDA's docs-layer types each have
a conventional id prefix.)

Title: the choice in one sentence ("Use Postgres for primary storage").
Description: include the alternatives you considered + the trade-offs +
a one-line "why we chose this".

## Tip

Good ADRs include the **reversibility** of the decision. "Easy to swap"
vs. "load-bearing — would require a migration" is information the next
maintainer needs.

## Verify

`aida-tutor verify` — looks for any `ADR-*` requirement.
