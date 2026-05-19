## Step 2 — capture one real intent

`greet` only ever greets in mixed case. Say you want a `--upper` flag that
shouts the greeting. That is a real intent — so capture it as a spec
*before* writing any code:

```
aida add --type functional --status approved --title "greet --upper flag shouts the greeting"
```

AIDA assigns it an id: `FR-1`, the first functional requirement in this
project. That id is now a stable handle — your code, your commits, and
your agent will all refer to this feature by it.

This is the habit AIDA is built on: *capture intent, then build*. Two
minutes of typing turns "something I want" into "something the project
knows it is supposed to have."
