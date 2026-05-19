//! Exercise 17 — `aida push`. trace:STORY-17 | ai:claude

use crate::exercise::{run, Exercise, VerifyResult};
use crate::verify::is_aida_initialized;
use std::path::Path;

pub struct E;

impl Exercise for E {
    fn id(&self) -> u32 { 17 }
    fn slug(&self) -> &'static str { "push" }
    fn title(&self) -> &'static str { "aida push — unified code + store push" }
    fn hint(&self) -> &'static str {
        "Run `aida push`. With no `origin` remote (which is normal for this tutorial) you'll see two \
         skip notes — one for the code leg, one for the store leg. With an `origin` configured, both \
         legs run. The single command saves you from forgetting `aida db sync --push` after `git push`."
    }
    // trace:STORY-20 | ai:claude
    fn hint_more(&self) -> Option<&'static str> {
        Some(
            "1. From `workspace/`, run `aida push`.\n\
             2. With no `origin` remote you'll see two skip notes — expected for this tutorial.\n\
             3. The single command ships code + store together when a remote does exist."
        )
    }
    // trace:STORY-20 | ai:claude
    fn hint_solution(&self) -> Option<&'static str> {
        Some("aida push")
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        if !is_aida_initialized(workspace) {
            return VerifyResult::Pending("complete exercise 01 first".into());
        }
        VerifyResult::Pass
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        // No `origin` in the demo workspace — push skips both legs
        // cleanly and exits 0 without prompting.
        run(workspace, "aida", &["push"])
    }
}
