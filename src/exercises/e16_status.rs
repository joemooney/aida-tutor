//! Exercise 16 — `aida status`. trace:STORY-16 | ai:claude

use crate::exercise::{run, Exercise, VerifyResult};
use crate::verify::is_aida_initialized;
use std::path::Path;

pub struct E;

impl Exercise for E {
    fn id(&self) -> u32 { 16 }
    fn slug(&self) -> &'static str { "status" }
    fn title(&self) -> &'static str { "aida status — read the project pulse" }
    fn hint(&self) -> &'static str {
        "Run `aida status`. The output has sections: Project (name + mode + path), Requirements (counts \
         by status), Cache (freshness), Sync (orphan branch ahead/behind), Recent activity, Scaffolding. \
         This is the first command to run when you sit down to a project — it tells you exactly what \
         needs your attention."
    }
    // trace:STORY-20 | ai:claude
    fn hint_more(&self) -> Option<&'static str> {
        Some(
            "1. From `workspace/`, run AIDA's one-word project-pulse command.\n\
             2. Read the sections: Project, Requirements, Cache, Sync, Recent activity.\n\
             3. That's the command to start any work session with."
        )
    }
    // trace:STORY-20 | ai:claude
    fn hint_solution(&self) -> Option<&'static str> {
        Some("aida status")
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        if !is_aida_initialized(workspace) {
            return VerifyResult::Pending("complete exercise 01 first".into());
        }
        VerifyResult::Pass
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        run(workspace, "aida", &["status"])
    }
}
