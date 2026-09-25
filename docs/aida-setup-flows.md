# AIDA Setup Flows

<!-- trace:STORY-47,STORY-48 | ai:codex -->

This guide is the maintainer-facing version of the tutor's setup lesson.
It focuses on AIDA 0.14 project bootstrap: starting a new project and
enabling AIDA in an existing one.

## New Project

Recommended full setup:

```bash
mkdir my-project
cd my-project
aida init --git-init --name my-project
aida add --type functional --status approved --title "First real requirement"
```

Equivalent explicit-git setup:

```bash
mkdir my-project
cd my-project
git init
aida init --name my-project
```

Full `aida init` creates the git-canonical requirement store on the
`aida-store` orphan branch, checks it out at `.aida-store/`, writes
`.aida/config.toml`, maintains a rebuildable `.aida/cache.db`, scaffolds
agent instructions and skills, writes `.mcp.json`, installs hooks, and
bootstraps starter roles unless you opt out.

Use `aida init --minimal` only for the markdown-only first look. It creates
`specs/` plus a runnable `aida why` demo, but no orphan store, cache, MCP,
skills, hooks, or roles. Run full `aida init` later when the project needs
the normal AIDA workflow.

Agent-scaffold choices:

- `aida init --agent both` is the default and scaffolds Claude Code and
  Codex surfaces.
- `aida init --agent claude` or `--agent codex` narrows the scaffold.
- `aida init --no-skills` keeps the project store but skips generated
  agent skills and commands.

Share the project by setting a remote and pushing both legs:

```bash
aida remote create
aida push
```

`aida push` pushes the code branch and the `aida-store` branch. `aida pull`
pulls both. Avoid treating `.aida/cache.db` as durable data; it is a local
projection that `aida cache rebuild` can recreate.

## Existing Project

Start at the repository root:

```bash
aida init --name existing-project
```

This does not require an empty repo. It adds AIDA's project scaffold around
the existing code. Review the resulting diff before committing, especially
files that overlap with existing agent guidance.

Expected project files and directories:

- `.aida/config.toml` — local project config.
- `.aida-store/` — linked worktree for the canonical requirement store.
- `.mcp.json` — local MCP server wiring for clients that read it.
- `CLAUDE.md` and `.claude/` — Claude Code instructions, commands, skills,
  settings, and hooks.
- `AGENTS.md` and `.codex/skills/` — Codex and MCP-compatible agent
  instructions and skills.
- `.antigravity/` — scaffolded when the installed AIDA template set includes
  Antigravity support.
- `.git/hooks/` — commit validation and store-pairing hooks.
- `.aida/cache.db` — rebuildable local SQLite cache.

Use `--commit-scaffold` when bootstrapping a clone of an already-initialized
AIDA project and you intentionally want AIDA to commit locally written
scaffold files. The safe default leaves those writes uncommitted so a clone
does not accidentally push a scaffold refresh to the shared branch.

Use `--force` only when you intend to overwrite existing scaffold files. For
routine drift, prefer:

```bash
aida scaffold status
aida scaffold diff
aida scaffold upgrade
```

Codex may still need manual MCP registration even when `.mcp.json` exists:

```bash
codex mcp add aida -- aida mcp-serve
```

Claude Code reads the project `.mcp.json` path in its normal project setup.
Other MCP clients should point at the same local command: `aida mcp-serve`.

Verification checklist:

```bash
aida status
aida list
aida store status
aida docs build
aida docs check
```

For multi-repo workspaces, prefer a shared sibling store:

```bash
aida init --sibling
aida init --attach --store-path ../aida-store
```

`--sibling` creates or uses a separate store repo. `--attach` joins an
existing store without changing its contents.
