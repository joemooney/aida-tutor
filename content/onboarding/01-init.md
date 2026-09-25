<!-- trace:STORY-47,STORY-48,STORY-50 | ai:codex -->

## Step 1 — give the project a memory

You selected this AIDA init profile for the tour: `{{scaffold_agents}}`.
From inside the scratch project, run the matching bootstrap:

```
cd workspace
{{init_command}}
```

{{init_note}}

This creates `.aida-store/` — a small store, kept on its own git branch,
that will hold every spec, decision, and code-to-spec link for this
project. It also writes `.aida/config.toml`, a rebuildable `.aida/cache.db`,
the selected agent instructions and skills when AIDA supports them, commit
hooks, and `.mcp.json`, the wiring that lets your coding agent read the
store later. That `.mcp.json` is the payoff in step 6 — leave it in place.

In a brand-new non-git directory you can add `--git-init` to the same
command. In an existing git repo, run the same agent-scoped init from the
repo root, review the scaffolded files, commit them deliberately, then keep
using the repo as usual. `aida init --minimal` is only the markdown-only
first-look mode; run full `aida init` when you want the store, MCP, hooks,
roles, and agent workflow.

`aida init` is a one-time act per project. Once it's done, the project has
somewhere to remember things — and the rest of this tour fills that memory
in.
