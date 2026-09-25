<!-- trace:STORY-49,STORY-54 | ai:codex -->

## Step 3 — hand FR-1 to {{agent_display}}

{{agent_prompt_intro}}

{{implement_action}}

`aida-tutor` is still the guide and verifier. The selected implementer,
`{{agent_display}}`, changes the code. For Codex, this command launches a
headless Codex run in the scratch workspace; you should see `greet.py`
change without editing it by hand.

Before the edit, the command runs `aida spec dryrun FR-1`. That is the
small advisor-style gate for this first tour: it checks whether the spec is
ready for an implementer before the AI starts changing files.

In this scratch project, `dryrun` may warn that FR-1 has no parent. That is
fine for the tour because this is a one-feature toy repo. In a production
burndown, the advisor would normally link ready work under an epic before
queueing it.

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
