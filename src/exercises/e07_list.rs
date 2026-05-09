//! Exercise 07 — `aida list` (read-only; we verify via "all 5 types exist").
//! trace:STORY-7 | ai:claude

use crate::exercise::{Exercise, VerifyResult};
use crate::verify::{is_aida_initialized, requirements_with_prefix};
use std::path::Path;

pub struct E;

impl Exercise for E {
    fn id(&self) -> u32 { 7 }
    fn slug(&self) -> &'static str { "list" }
    fn title(&self) -> &'static str { "aida list — see all five reqs at once" }
    fn hint(&self) -> &'static str {
        "Run `aida list` (no args). You should see the 5 reqs you've created — visually grouped by id. \
         The display order is most-recently-modified first. META rows are hidden by default; pass \
         `--include-meta` if you want to peek under the hood."
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        if !is_aida_initialized(workspace) {
            return VerifyResult::Pending("complete exercise 01 first".into());
        }
        for prefix in ["VIS", "PRIN", "ADR", "FR", "BUG"] {
            if requirements_with_prefix(workspace, prefix).is_empty() {
                return VerifyResult::Pending(
                    format!("can't verify aida list yet — exercise needs all 5 types present. Missing: {}", prefix)
                );
            }
        }
        // Best-effort: this exercise has no concrete proof-of-execution
        // beyond "by now you should have run it". Pass once the prerequisite
        // state exists. trace:PRIN-1 — verifier inspects state, not history.
        VerifyResult::Pass
    }
}
