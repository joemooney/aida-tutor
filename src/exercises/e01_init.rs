//! Exercise 01 — `aida init`. trace:STORY-1 | ai:claude

use crate::exercise::{Exercise, VerifyResult};
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
}
