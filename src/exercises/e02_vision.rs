//! Exercise 02 — capture project vision (VIS-1). trace:STORY-2 | ai:claude

use crate::exercise::{Exercise, VerifyResult};
use crate::verify::{is_aida_initialized, requirements_with_prefix};
use std::path::Path;

pub struct E;

impl Exercise for E {
    fn id(&self) -> u32 { 2 }
    fn slug(&self) -> &'static str { "vision" }
    fn title(&self) -> &'static str { "capture project vision (VIS-1)" }
    fn hint(&self) -> &'static str {
        "Add a requirement of type `vision` whose --title is the project's intent in one sentence. \
         Set --status approved (visions don't sit in draft — they're declarations). \
         The id will land as VIS-1 because no other vision exists yet."
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        if !is_aida_initialized(workspace) {
            return VerifyResult::Pending(
                "complete exercise 01 first (no AIDA store yet)".into()
            );
        }
        let visions = requirements_with_prefix(workspace, "VIS");
        if visions.is_empty() {
            return VerifyResult::Pending(
                "no VIS-* requirement found — try `aida add --type vision --title \"...\" --status approved`".into()
            );
        }
        // Optional: warn if the user left it as draft, but accept either.
        let approved = visions.iter().any(|r| {
            r.status.as_deref().map(|s| s.eq_ignore_ascii_case("approved") || s.eq_ignore_ascii_case("Approved")).unwrap_or(false)
        });
        if !approved {
            return VerifyResult::Fail(
                "VIS-* found but its status is not `approved`. Visions are declarations — \
                 use `aida edit VIS-1 --status approved` to advance it.".into()
            );
        }
        VerifyResult::Pass
    }
}
