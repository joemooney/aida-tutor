//! Exercise 06 — capture a bug (BUG). trace:STORY-6 | ai:claude

use crate::exercise::{run, Exercise, VerifyResult};
use crate::verify::{is_aida_initialized, requirements_with_prefix};
use std::path::Path;

pub struct E;

impl Exercise for E {
    fn id(&self) -> u32 { 6 }
    fn slug(&self) -> &'static str { "bug" }
    fn title(&self) -> &'static str { "capture a bug (BUG)" }
    fn hint(&self) -> &'static str {
        "Add a requirement of type `bug`. AIDA bugs can be filed BEFORE the code that contains the bug — \
         capturing the reasoning is a feature, not a misuse. Pick anything concrete: 'leap-year edge case', \
         'rate limiter ignores burst', etc. Status approved, priority your-call."
    }
    // trace:STORY-20 | ai:claude
    fn hint_more(&self) -> Option<&'static str> {
        Some(
            "1. Run `aida add --type bug` — a bug can be filed before the buggy code even exists.\n\
             2. Title it concretely; `--status approved`, priority your call.\n\
             3. `aida list --type bug` confirms BUG-1."
        )
    }
    // trace:STORY-20 | ai:claude
    fn hint_solution(&self) -> Option<&'static str> {
        Some("aida add --type bug --status approved --title \"Leap-year edge case in the date parser\"")
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        if !is_aida_initialized(workspace) {
            return VerifyResult::Pending("complete exercise 01 first".into());
        }
        let bugs = requirements_with_prefix(workspace, "BUG");
        if bugs.is_empty() {
            return VerifyResult::Pending(
                "no BUG-* requirement found — try `aida add --type bug ...`".into()
            );
        }
        VerifyResult::Pass
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        run(workspace, "aida", &[
            "add", "--type", "bug", "--status", "approved",
            "--title", "Leap-year edge case in the date parser",
        ])
    }
}
