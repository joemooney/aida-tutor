## Goal

Bootstrap an AIDA store inside `workspace/`.

## Why

Every AIDA project starts with `aida init`. It does three things:

1. Creates a separate git branch (`aida-store`) and a worktree at
   `.aida-store/`. That branch is the canonical home for every requirement,
   feature, bug, decision, and trace link in your project — kept on its own
   branch so it travels with the repo without polluting your code branches.
2. Drops a small `.aida/` directory with a config file and a SQLite cache
   that makes `aida list` and `aida search` fast.
3. Scaffolds a `.claude/` directory with skills, slash commands, hooks,
   plus a `CLAUDE.md` that auto-imports AIDA conventions for any agent
   running in this repo.

After this exercise you have a real AIDA project. From here, every other
exercise builds on it.

## What to do

In a shell, `cd` into the `workspace/` directory and run `aida init`.

(The `workspace/` directory was created empty for you by `aida-tutor reset`
— it's already a git repo with one commit, ready to receive AIDA.)

## What you'll see

A line for the orphan branch creation, a line for `metadata.yaml`, an
"acquired node id" line (the short id AIDA picks for you — from
`~/.aida/preferences.toml` if you've set a preferred one, otherwise
derived for you), and a final `AIDA initialized ✓`. The exact node id
doesn't matter — the verifier only checks the store exists.

## Verify

Once you've run init, come back and run `aida-tutor verify`.
