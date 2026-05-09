# aida-tutor

Hands-on tutorial for **AIDA** (`aida-cli`), built in the spirit of
[rustlings](https://github.com/rust-lang/rustlings).

Walks you through the full AIDA workflow exercise-by-exercise:

```
01. aida init                       09. edit FR-1 → in-progress
02. capture vision (VIS-1)          10. write code with trace:FR-1 comment
03. capture principle (PRIN-1)      11. commit with the AIDA format
04. capture decision (ADR-1)        12. aida docs build
05. capture feature (FR-1)          13. aida search "<keyword>"
06. capture bug (BUG-1)             14. complete FR-1 + add a comment
07. aida list                       15. aida show <id> --comments
08. aida show <id>                  16. aida status (project pulse)
                                    17. aida push (unified)
```

Each exercise has a goal, a why, instructions, and a tip. The tutor
verifies your work by inspecting on-disk state — it never edits your
files. If a verifier passes, you actually did the thing.

## Quickstart

```bash
git clone <this repo>
cd aida-tutor
cargo build --release
./target/release/aida-tutor reset --yes      # bootstrap workspace/
./target/release/aida-tutor show              # see the current exercise
./target/release/aida-tutor hint              # get a one-paragraph nudge
./target/release/aida-tutor verify            # check your work
./target/release/aida-tutor list              # see all exercises + state
./target/release/aida-tutor progress          # 5/17 done — 29%
```

You'll need `aida` on `PATH` for the exercises themselves. `aida-tutor`
spawns it as a subprocess to give you the real CLI surface.

## Layout

```
aida-tutor/
├── Cargo.toml
├── README.md            you are here
├── CLAUDE.md            project context for AI agents
├── content/             exercise descriptions (markdown — edit freely)
│   ├── 01-init.md
│   ├── 02-vision.md
│   └── ...
├── src/                 Rust binary
│   ├── main.rs          CLI dispatch
│   ├── exercise.rs      trait + verify result types
│   ├── exercises/       one module per exercise (verifier + hint)
│   ├── verify.rs        on-disk state inspection helpers
│   └── progress.rs      progress persistence
├── .aida/               tutor's own AIDA store (yes — we dogfood)
├── .aida-store/         orphan worktree
├── workspace/           where YOU work the exercises (gitignored)
└── .aida-tutor-progress.toml   your completion record
```

## Adding new exercises

1. Write the description as `content/NN-slug.md`.
2. Implement the verifier as `src/exercises/eNN_slug.rs`.
3. Register it in `src/exercises/mod.rs`.
4. File it as a STORY in the tutor's own AIDA store (`aida add --type
   story ...`).

The principle (PRIN-1, in this repo's store) is that **verifiers
inspect, never patch**. Read the workspace, never write to it.

## License

MIT OR Apache-2.0
