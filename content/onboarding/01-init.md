## Step 1 — give the project a memory

From inside the scratch project, run AIDA's one-word bootstrap:

```
cd workspace
aida init
```

This creates `.aida-store/` — a small store, kept on its own git branch,
that will hold every spec, decision, and code-to-spec link for this
project. It also scaffolds `.mcp.json`, the wiring that lets your coding
agent read the store later. That `.mcp.json` is the payoff in step 6 —
leave it in place.

`aida init` is a one-time act per project. Once it's done, the project has
somewhere to remember things — and the rest of this tour fills that memory
in.
