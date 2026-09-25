<!-- trace:STORY-47,STORY-48 | ai:codex -->

## Goal

Bootstrap an AIDA store inside `workspace/`.

## Why

Most real AIDA projects start with full `aida init`. It does five
practical things:

1. Creates a separate git branch (`aida-store`) and a worktree at
   `.aida-store/`. That branch is the canonical home for every requirement,
   feature, bug, decision, and trace link in your project — kept on its own
   branch so it travels with the repo without polluting your code branches.
2. Drops a small `.aida/` directory with a config file and a SQLite cache
   that makes `aida list` and `aida search` fast. The cache is rebuildable;
   the orphan branch is canonical.
3. Scaffolds agent-facing project instructions and skills. By default AIDA
   uses `--agent both`, so Claude Code gets `CLAUDE.md` plus `.claude/`,
   and Codex gets `AGENTS.md` plus `.codex/skills/`.
4. Writes `.mcp.json`, the local MCP wiring that lets compatible coding
   agents talk to `aida mcp-serve` instead of scraping terminal output.
5. Installs commit hooks and starter roles unless you opt out with flags
   such as `--no-hooks`, `--no-skills`, or `--no-roles`.

After this exercise you have a real AIDA project. From here, every other
exercise builds on it.

## New-project choices

For a normal new project, create a directory and run one of these:

```bash
git init
aida init --name my-project
```

or, when the directory is not a git repo yet:

```bash
aida init --git-init --name my-project
```

For a quick markdown-only trial, `aida init --minimal` creates just a
`specs/` folder and a runnable `aida why` demo. That is useful for a
60-second first look, but it is not the full agent-collaboration setup:
no orphan store, cache, MCP, skills, hooks, or roles until you run full
`aida init` later.

For multi-repo workspaces, `aida init --sibling` creates a sibling store
repo, and `aida init --attach --store-path <path>` joins an existing
shared store. This single-repo tutorial uses the default orphan-branch
store because it is the common path.

## Existing-project choices

Running `aida init` in an existing git repo is expected. Review the
scaffolded files like any other project change, then commit them when the
team is ready. If you are bootstrapping a clone of an already-initialized
AIDA project, AIDA leaves local scaffold writes uncommitted by default;
`--commit-scaffold` opts into committing them deliberately. Use `--force`
only when you really intend to overwrite existing scaffold files.

## What to do

In a shell, `cd` into the `workspace/` directory and run `aida init`.

(The `workspace/` directory was created empty for you by `aida-tutor reset`
— it's already a git repo with one commit, ready to receive AIDA.)

## What you'll see

A line for the orphan branch creation, a line for `metadata.yaml`, an
"acquired node id" line (the short id AIDA picks for you — from
`~/.aida/preferences.toml` if you've set a preferred one, otherwise
derived for you), scaffold output for agent instructions and hooks, and a
final `AIDA initialized ✓`. The exact node id doesn't matter — the
verifier only checks the store exists.

## Verify

Once you've run init, come back and run `aida-tutor verify`.
