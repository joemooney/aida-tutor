# aida-tutor

Hands-on tutorial for **AIDA** (`aida-cli`), built in the spirit of
[rustlings](https://github.com/rust-lang/rustlings).

`aida-tutor` has two paths:

- `aida-tutor onboard` — a 15-minute first-contact tour: initialize a
  scratch project, capture one requirement, trace code to it, commit it,
  and show the link back through AIDA.
- The 36-exercise track — the fuller workflow: project setup, capture,
  trace comments, AIDA-format commits, docs, search/status, push/pull,
  the distributed store, roles and queues, relationships, scoped
  sessions, review, plans, store audit, and MCP.

The tutor verifies your work by inspecting on-disk state and selected
command output. It never edits your exercise files. If a verifier passes,
you actually did the thing.

## Quickstart

```bash
git clone <this repo>
cd aida-tutor
cargo build --release
./target/release/aida-tutor shell            # recommended first run
./target/release/aida-tutor onboard          # full onboarding lesson
./target/release/aida-tutor next             # terse next action
./target/release/aida-tutor reset --yes      # bootstrap workspace/
./target/release/aida-tutor show             # see the current exercise
./target/release/aida-tutor hint             # get a one-paragraph nudge
./target/release/aida-tutor verify           # check your work
./target/release/aida-tutor list             # see all exercises + state
./target/release/aida-tutor progress         # e.g. 5/36 done - 14%
```

You'll need `aida` on `PATH` for the exercises themselves. `aida-tutor`
spawns it as a subprocess to give you the real CLI surface.

## Starting AIDA Projects

For a new production project, the normal path is full git-canonical init:

```bash
mkdir my-project
cd my-project
aida init --git-init --name my-project
```

If the repo already exists, run `aida init` from the repository root and
review the scaffolded files like any other change. Current AIDA scaffolds
the requirement store, local cache, MCP config, agent instructions, skills,
roles, and commit hooks. See
[docs/aida-setup-flows.md](docs/aida-setup-flows.md) for the new-project
and existing-project checklists.

`aida init --minimal` is a quick markdown-only trial. It is useful for a
first look, but it does not enable the full store, MCP, hooks, skills, or
agent workflow until you run full `aida init`.

## Layout

```
aida-tutor/
├── Cargo.toml
├── README.md            you are here
├── CLAUDE.md            project context for AI agents
├── AGENTS.md            Codex / MCP-compatible agent guidance
├── content/             exercise descriptions (markdown — edit freely)
├── docs/                maintainer notes and setup guides
├── src/                 Rust binary
│   ├── main.rs          CLI dispatch
│   ├── exercise.rs      trait + verify result types
│   ├── exercises/       one module per exercise (verifier + hint)
│   ├── verify.rs        on-disk state inspection helpers
│   └── progress.rs      progress persistence
├── .aida/               tutor's own AIDA store cache/config
├── .aida-store/         orphan worktree for the tutor's requirements
├── workspace/           where YOU work the exercises (gitignored)
└── .aida-tutor-progress.toml   your completion record
```

## Adding New Exercises

1. Write the description as `content/NN-slug.md`.
2. Implement the verifier as `src/exercises/eNN_slug.rs`.
3. Register it in `src/exercises/mod.rs`.
4. File it as a STORY in the tutor's own AIDA store:
   `aida add --type story ...`.

The principle (PRIN-1, in this repo's store) is that **verifiers inspect,
never patch**. Read the workspace, never write to it.

## License

MIT OR Apache-2.0
