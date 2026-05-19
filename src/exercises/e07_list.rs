//! Exercise 07 — `aida list` (read-only; we verify via "all 5 types exist").
//! trace:STORY-7 | ai:claude

use crate::exercise::{run, verify_invocation, Exercise, VerifyResult};
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
    // trace:STORY-20 | ai:claude
    fn hint_more(&self) -> Option<&'static str> {
        Some(
            "1. You should have one VIS, one PRIN, one ADR, one FR and one BUG captured by now.\n\
             2. Run AIDA's listing command with no arguments.\n\
             3. Confirm all five rows appear, then `aida-tutor verify`."
        )
    }
    // trace:STORY-20 | ai:claude
    fn hint_solution(&self) -> Option<&'static str> {
        Some("aida list")
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
        // Read-only command: with the invocation-logging wrapper opted
        // in (`aida-tutor wrapper`) require a real `aida list` run;
        // without it, the 5-type prerequisite above is the best signal
        // we have. trace:PRIN-1 — verifier inspects state, not history.
        // trace:STORY-22 | ai:claude
        verify_invocation(
            workspace,
            "list",
            &[],
            "all 5 types are present, but the invocation wrapper shows no `aida list` yet — run it",
        )
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        run(workspace, "aida", &["list"])
    }
}
