# AIDA Tutor

AIDA Tutor is a Rust terminal launchpad for people learning and using the
AIDA requirements workflow. It combines a guided first-contact tour, a
41-exercise track with on-disk verification, an AIDA shell, and reference
menus for terminology, FAQs, and commands.

`src/main.rs` dispatches CLI commands. `src/onboarding.rs` implements the
launchpad, tutorial menus, and interactive shells. Exercises live under
`src/exercises/`, with descriptions in `content/`; `src/verify.rs` inspects
workspace state and `src/progress.rs` persists completion. Verifiers inspect
the learner's work rather than modifying it.

`src/commands.rs` loads the installed AIDA command catalog using
`aida help commands --hidden --json`. It filters names, descriptions, and
flags and loads each selected command's live `--help`, caching it for the
current visit. The terminal view colors help sections and supports
previous/next navigation through filtered results independently of scrolling.
Crossterm manages the interactive screen and input; redirected output prints
the filtered catalog. Older AIDA versions without the JSON catalog cannot
provide this browser.

Requirements and implementation history live in the project's AIDA store.
README.md covers startup, browser controls, setup flows, and maintenance of
the separate upstream AIDA help snapshot.
