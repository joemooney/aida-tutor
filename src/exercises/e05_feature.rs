//! Exercise 05 — capture a feature (FR). trace:STORY-5 | ai:claude

use crate::exercise::{run, Exercise, VerifyResult};
use crate::verify::{is_aida_initialized, requirements_with_prefix};
use std::path::Path;

pub struct E;

impl Exercise for E {
    fn id(&self) -> u32 { 5 }
    fn slug(&self) -> &'static str { "feature" }
    fn title(&self) -> &'static str { "capture a feature (FR)" }
    fn hint(&self) -> &'static str {
        "Add a requirement of type `functional` — a behavior the project must do. \
         (Type is `functional`; the prefix `FR` is short for Functional Requirement.) \
         Status approved, priority high. Pick anything concrete: 'parse JSON input', 'persist user prefs', etc."
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        if !is_aida_initialized(workspace) {
            return VerifyResult::Pending("complete exercise 01 first".into());
        }
        let frs = requirements_with_prefix(workspace, "FR");
        if frs.is_empty() {
            return VerifyResult::Pending(
                "no FR-* requirement found — try `aida add --type functional ...`".into()
            );
        }
        let approved = frs.iter().any(|r| {
            r.status.as_deref().map(|s| s.eq_ignore_ascii_case("approved") || s.eq_ignore_ascii_case("Approved")).unwrap_or(false)
        });
        if !approved {
            return VerifyResult::Fail(
                "FR-* found but its status is not `approved`. \
                 Set --status approved when you `aida add`, or `aida edit FR-1 --status approved` after the fact.".into()
            );
        }
        VerifyResult::Pass
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        run(workspace, "aida", &[
            "add", "--type", "functional", "--status", "approved",
            "--priority", "high",
            "--title", "Parse JSON task input from stdin",
        ])
    }
}
