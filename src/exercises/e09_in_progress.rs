//! Exercise 09 — flip FR to in-progress. trace:STORY-9 | ai:claude

use crate::exercise::{demo_spec_id, run, Exercise, VerifyResult};
use crate::verify::{is_aida_initialized, requirements_with_prefix};
use std::path::Path;

pub struct E;

impl Exercise for E {
    fn id(&self) -> u32 { 9 }
    fn slug(&self) -> &'static str { "in-progress" }
    fn title(&self) -> &'static str { "edit FR-1 to in-progress" }
    fn hint(&self) -> &'static str {
        "Run `aida edit <FR-id> --status in-progress`. This is the lightweight 'I'm picking it up' signal. \
         (Inside Claude Code you'd usually run `/aida-pickup` instead — it does this from the next-queued \
         item — but for now we'll do it manually.)"
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        if !is_aida_initialized(workspace) {
            return VerifyResult::Pending("complete exercise 01 first".into());
        }
        let frs = requirements_with_prefix(workspace, "FR");
        if frs.is_empty() {
            return VerifyResult::Pending("no FR-* req present yet (exercise 05)".into());
        }
        let any_in_progress = frs.iter().any(|r| {
            r.status
                .as_deref()
                .map(|s| {
                    let s = s.to_lowercase();
                    s == "in-progress" || s == "inprogress" || s == "in progress"
                })
                .unwrap_or(false)
        });
        if !any_in_progress {
            return VerifyResult::Pending(
                "no FR-* is in-progress yet — try `aida edit <FR-id> --status in-progress`".into()
            );
        }
        VerifyResult::Pass
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        let fr = demo_spec_id(workspace, "FR")?;
        run(workspace, "aida", &["edit", &fr, "--status", "in-progress"])
    }
}
