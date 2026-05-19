## Step 3 — hand FR-1 to your agent

Now let your coding agent do the work. In Claude Code, ask it plainly —
something like:

    Implement FR-1 in greet.py: add a --upper flag that uppercases the
    greeting. Leave a "# trace:FR-1 | ai:claude" comment on the line that
    implements it.

The agent edits `greet.py` to add the flag and — this is the part that
matters — leaves a **trace comment** next to the code it wrote:

```
# trace:FR-1 | ai:claude
parser.add_argument("--upper", action="store_true", help="shout it")
```

That one comment is the durable link from code back to spec. You don't
keep a spreadsheet mapping features to files — the comment lives in the
source, travels with every copy of it, and AIDA reads it directly.

No coding agent handy? Add the flag and the `# trace:FR-1` comment
yourself — the tour works the same either way.
