//! Exercise 03 — `aida list` (read-only; we verify the captured feature
//! shows up). trace:STORY-7 | ai:claude

use crate::exercise::{run, verify_invocation, Exercise, VerifyResult};
use crate::verify::{is_aida_initialized, requirements_with_prefix};
use std::path::Path;

pub struct E;

impl Exercise for E {
    fn id(&self) -> u32 {
        3
    }
    fn slug(&self) -> &'static str {
        "list"
    }
    fn title(&self) -> &'static str {
        "aida list — see your captured work"
    }
    fn hint(&self) -> &'static str {
        "Run `aida list` (no args). You should see FR-1, the feature you just captured — \
         its type, status, priority, and title in one row. META rows (seeded by `aida init`) \
         are hidden by default; pass `--include-meta` if you want to peek under the hood."
    }
    // trace:STORY-20 | ai:claude
    fn hint_more(&self) -> Option<&'static str> {
        Some(
            "1. FR-1 is the feature you captured in exercise 02.\n\
             2. Run AIDA's listing command with no arguments.\n\
             3. Confirm FR-1 appears, then `aida-tutor verify`.",
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
        if requirements_with_prefix(workspace, "FR").is_empty() {
            return VerifyResult::Pending(
                "no FR-* to list yet — capture the feature in exercise 02 first".into(),
            );
        }
        // Read-only command: with the invocation-logging wrapper opted
        // in (`aida-tutor wrapper`) require a real `aida list` run;
        // without it, FR-1's presence is the best signal we have.
        // trace:PRIN-1 — verifier inspects state, not history.
        // trace:STORY-22 | ai:claude
        verify_invocation(
            workspace,
            "list",
            &[],
            "FR-1 is present, but the invocation wrapper shows no `aida list` yet — run it",
        )
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        run(workspace, "aida", &["list"])
    }
}
