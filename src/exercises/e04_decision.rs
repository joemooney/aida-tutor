//! Exercise 04 — capture a decision (ADR). trace:STORY-4 | ai:claude

use crate::exercise::{run, Exercise, VerifyResult};
use crate::verify::{is_aida_initialized, requirements_with_prefix};
use std::path::Path;

pub struct E;

impl Exercise for E {
    fn id(&self) -> u32 { 4 }
    fn slug(&self) -> &'static str { "decision" }
    fn title(&self) -> &'static str { "capture a decision (ADR)" }
    fn hint(&self) -> &'static str {
        "Add a requirement of type `decision` (NOT `adr` — the type is `decision`, the prefix `ADR` \
         is conventional). The description should record the choice + the trade-offs that drove it. \
         An ADR's value comes from why-we-chose-it, not what-we-chose."
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        if !is_aida_initialized(workspace) {
            return VerifyResult::Pending("complete exercise 01 first".into());
        }
        let adrs = requirements_with_prefix(workspace, "ADR");
        if adrs.is_empty() {
            return VerifyResult::Pending(
                "no ADR-* requirement found — try `aida add --type decision ...`".into()
            );
        }
        VerifyResult::Pass
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        run(workspace, "aida", &[
            "add", "--type", "decision", "--status", "approved",
            "--title", "Persist tasks in SQLite, not flat files",
        ])
    }
}
