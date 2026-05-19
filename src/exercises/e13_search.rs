//! Exercise 13 — `aida search`. trace:STORY-13 | ai:claude

use crate::exercise::{run, Exercise, VerifyResult};
use crate::verify::is_aida_initialized;
use std::path::Path;

pub struct E;

impl Exercise for E {
    fn id(&self) -> u32 { 13 }
    fn slug(&self) -> &'static str { "search" }
    fn title(&self) -> &'static str { "aida search '<keyword>' — cross-cutting find" }
    fn hint(&self) -> &'static str {
        "Run `aida search` with a word from any of your req titles or descriptions. The search hits \
         BOTH titles and bodies via FTS5, sub-millisecond. Useful when you remember 'something about \
         the auth flow' but not the id."
    }
    // trace:STORY-20 | ai:claude
    fn hint_more(&self) -> Option<&'static str> {
        Some(
            "1. Pick a word that appears in one of your requirement titles — `JSON`, say.\n\
             2. Run `aida search` with that word.\n\
             3. The hit list spans both titles and descriptions."
        )
    }
    // trace:STORY-20 | ai:claude
    fn hint_solution(&self) -> Option<&'static str> {
        Some("aida search JSON")
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        if !is_aida_initialized(workspace) {
            return VerifyResult::Pending("complete exercise 01 first".into());
        }
        // Read-only command — pass on prerequisite state.
        VerifyResult::Pass
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        run(workspace, "aida", &["search", "JSON"])
    }
}
