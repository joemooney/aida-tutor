## Step 2 — capture one real intent

`greet` only ever greets in mixed case. Say you want a `--upper` flag that
shouts the greeting. That is a real intent — so capture it as a spec
*before* writing any code:

```
aida add --type functional --status approved --title "greet --upper flag shouts the greeting" --description "Add a --upper flag to greet.py that prints the greeting in uppercase when provided. Acceptance: python3 greet.py --upper World prints HELLO, WORLD! and python3 greet.py World still prints Hello, World!"
```

AIDA assigns it an id: `FR-1`, the first functional requirement in this
project. That id is now a stable handle — your code, your commits, and
your agent will all refer to this feature by it.

The description is intentionally short, but it includes acceptance
criteria. That gives the advisor/readiness pass and the implementer enough
signal to work from the issue instead of guessing.

This is the habit AIDA is built on: *capture intent, make it clear enough
to build, then let the agent implement it*. Two minutes of typing turns
"something I want" into "something the project knows it is supposed to
have."
