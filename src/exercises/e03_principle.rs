//! Exercise 03 — capture a principle (PRIN). trace:STORY-3 | ai:claude

use crate::exercise::{run, Exercise, VerifyResult};
use crate::verify::{is_aida_initialized, requirements_with_prefix};
use std::path::Path;

pub struct E;

impl Exercise for E {
    fn id(&self) -> u32 { 3 }
    fn slug(&self) -> &'static str { "principle" }
    fn title(&self) -> &'static str { "capture a principle (PRIN)" }
    fn hint(&self) -> &'static str {
        "Add a requirement of type `principle` — a non-negotiable rule the project commits to. \
         Examples: \"Default to UTC in storage\" or \"Errors are values, not exceptions\". \
         Status approved, priority high."
    }
    // trace:STORY-20 | ai:claude
    fn hint_more(&self) -> Option<&'static str> {
        Some(
            "1. Run `aida add --type principle` with `--status approved --priority high`.\n\
             2. Title it as a non-negotiable rule — e.g. \"Errors are values, not exceptions\".\n\
             3. `aida list --type principle` shows it as PRIN-1."
        )
    }
    // trace:STORY-20 | ai:claude
    fn hint_solution(&self) -> Option<&'static str> {
        Some("aida add --type principle --status approved --priority high --title \"Errors are values, not exceptions\"")
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        if !is_aida_initialized(workspace) {
            return VerifyResult::Pending("complete exercise 01 first".into());
        }
        let prins = requirements_with_prefix(workspace, "PRIN");
        if prins.is_empty() {
            return VerifyResult::Pending(
                "no PRIN-* requirement found — try `aida add --type principle ...`".into()
            );
        }
        VerifyResult::Pass
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        run(workspace, "aida", &[
            "add", "--type", "principle", "--status", "approved",
            "--priority", "high",
            "--title", "Errors are values, not exceptions",
        ])
    }
}
