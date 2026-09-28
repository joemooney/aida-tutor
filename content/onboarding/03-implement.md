<!-- trace:STORY-49,STORY-54 | ai:codex -->

## Step 3 — hand FR-1 to {{agent_display}}

{{agent_prompt_intro}}

{{implement_action}}

`aida-tutor` is still the guide and verifier. The selected implementer,
`{{agent_display}}`, changes the code. For Codex, this command launches a
headless Codex run in the scratch workspace; you should see `greet.py`
change without editing it by hand.

Before the edit, the command checks `aida spec dryrun FR-1`. That is the
small advisor-style gate for this first tour: it checks whether the spec is
ready for an implementer before the AI starts changing files. If the only
warning is the missing parent, the tutor explains that this standalone tour
does not queue the work and continues directly; it will not send you around
the same dryrun loop. Other readiness failures stop the action and tell you
what to fix before asking for the next action again.

The implementer should add the flag and — this is the part that matters —
leave a **trace comment** next to the code:

```
# trace:FR-1 | ai:{{trace_tool}}
if args.upper:
    greeting = greeting.upper()
```

That one comment is the durable link from code back to spec. You don't
keep a spreadsheet mapping features to files — the comment lives in the
source, travels with every copy of it, and AIDA reads it directly.

If you chose the manual path, add the flag and the `# trace:FR-1 |
ai:human` comment yourself — the tour works the same either way.
