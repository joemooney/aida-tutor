//! Exercise 31 — the `Aida-Store:` commit trailer: every code commit is
//! pinned to the orphan-store SHA it was written against. Cluster 5
//! (code review + commit pairing). trace:STORY-29 | ai:claude

use crate::exercise::{demo_spec_id, run, Exercise, VerifyResult};
use crate::verify::{is_aida_initialized, last_commit_message_for_path, path_exists};
use std::path::Path;

pub struct E;

/// The file the learner commits to trigger — and prove — the trailer.
/// The verifier inspects the commit that last touched this exact path,
/// so it checks the learner's commit, not an older one. trace:STORY-29
const ARTIFACT: &str = "PAIRING.md";

impl Exercise for E {
    fn id(&self) -> u32 { 31 }
    fn slug(&self) -> &'static str { "commit-pair-trailer" }
    fn title(&self) -> &'static str { "the Aida-Store: trailer — commits pinned to the store" }
    fn hint(&self) -> &'static str {
        "`aida init` installed a `prepare-commit-msg` hook back in exercise 01. Every time you \
         `git commit`, it appends an `Aida-Store: <sha>` trailer pinning the orphan store's \
         HEAD — so an old code commit always knows which store version it was written against. \
         You don't run anything to enable it; just commit. Create `workspace/PAIRING.md`, \
         `git add` it, and `git commit` with a normal AIDA-format message — then `git log -1` \
         shows the trailer and `aida store status` shows code and store aligned."
    }
    // trace:STORY-20 | ai:claude
    fn hint_more(&self) -> Option<&'static str> {
        Some(
            "1. Create `workspace/PAIRING.md` with any content.\n\
             2. `git add PAIRING.md`, then `git commit` it with a normal AIDA-format message.\n\
             3. `git log -1` shows the `Aida-Store:` trailer the hook appended on its own."
        )
    }
    // trace:STORY-20 | ai:claude
    fn hint_solution(&self) -> Option<&'static str> {
        Some(
            "# from inside workspace/:\n\
             echo \"# Code <-> store pairing\" > PAIRING.md\n\
             git add PAIRING.md\n\
             git commit -m \"[AI:claude] docs(pairing): note the store trailer (FR-1)\""
        )
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        if !is_aida_initialized(workspace) {
            return VerifyResult::Pending("complete exercise 01 first".into());
        }
        let Some(msg) = last_commit_message_for_path(workspace, ARTIFACT) else {
            return VerifyResult::Pending(if path_exists(workspace, ARTIFACT) {
                format!(
                    "`{ARTIFACT}` exists but isn't committed yet — run `git add {ARTIFACT}` \
                     then `git commit` it with an AIDA-format message."
                )
            } else {
                format!(
                    "no `{ARTIFACT}` yet — create `workspace/{ARTIFACT}`, then `git add` and \
                     `git commit` it."
                )
            });
        };
        // The hook appends `Aida-Store: <40-char-sha>` as a trailer line.
        // Match it leniently (an abbreviated sha still proves the pairing).
        let trailer = regex::Regex::new(r"(?m)^Aida-Store: [0-9a-f]{7,40}\s*$").unwrap();
        if !trailer.is_match(&msg) {
            return VerifyResult::Fail(format!(
                "the commit that added `{ARTIFACT}` has no `Aida-Store:` trailer — it was made \
                 with `--no-verify`, or the `prepare-commit-msg` hook is missing. Re-commit \
                 normally, or reinstall the hook with `aida store install-hook`."
            ));
        }
        VerifyResult::Pass
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        // The prepare-commit-msg hook (installed by `aida init` in
        // exercise 01) adds the `Aida-Store:` trailer on its own — the
        // demo just makes a normal commit and the trailer rides along.
        let fr = demo_spec_id(workspace, "FR")?;
        std::fs::write(
            workspace.join(ARTIFACT),
            "# Code <-> store pairing\n\nThis file's commit carries an `Aida-Store:` trailer.\n",
        )?;
        run(workspace, "git", &["add", ARTIFACT])?;
        let msg = format!("[AI:claude] docs(pairing): note the store trailer ({fr})");
        run(workspace, "git", &["commit", "-m", &msg])
    }
}
