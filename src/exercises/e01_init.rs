//! Exercise 01 — `aida init`. trace:STORY-1 | ai:claude

use crate::exercise::{run, Exercise, VerifyResult};
use crate::verify::{is_aida_initialized, path_exists};
use std::path::Path;

pub struct E;

impl Exercise for E {
    fn id(&self) -> u32 { 1 }
    fn slug(&self) -> &'static str { "init" }
    fn title(&self) -> &'static str { "aida init in an empty repo" }
    fn hint(&self) -> &'static str {
        "Inside the workspace/ directory, run a command that bootstraps an AIDA store. \
         You'll know it worked when `.aida-store/` and `.aida/config.toml` appear."
    }
    // trace:STORY-20 | ai:claude
    fn hint_more(&self) -> Option<&'static str> {
        Some(
            "1. Open a shell and `cd` into the `workspace/` directory.\n\
             2. Run AIDA's one-word bootstrap subcommand — the one that scaffolds a new store.\n\
             3. Confirm `.aida-store/` and `.aida/config.toml` now exist, then `aida-tutor verify`."
        )
    }
    // trace:STORY-20 | ai:claude
    fn hint_solution(&self) -> Option<&'static str> {
        Some("aida init")
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        if !workspace.exists() {
            return VerifyResult::Pending(
                "workspace/ doesn't exist yet — `aida-tutor reset` will create it for you.".into()
            );
        }
        if !path_exists(workspace, ".git") {
            return VerifyResult::Pending(
                "workspace/ isn't a git repo. `aida init` requires a git repo to attach the orphan store.".into()
            );
        }
        if is_aida_initialized(workspace) {
            VerifyResult::Pass
        } else {
            VerifyResult::Pending(
                "no AIDA store detected yet — try `aida init` from inside workspace/".into()
            )
        }
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        run(workspace, "aida", &["init"])
    }
}
