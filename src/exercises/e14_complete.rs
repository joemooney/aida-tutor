//! Exercise 14 — close the loop: complete + comment. trace:STORY-14 | ai:claude

use crate::exercise::{demo_spec_id, run, Exercise, VerifyResult};
use crate::verify::{is_aida_initialized, requirements_with_prefix};
use std::path::Path;

pub struct E;

impl Exercise for E {
    fn id(&self) -> u32 { 14 }
    fn slug(&self) -> &'static str { "complete" }
    fn title(&self) -> &'static str { "edit FR-1 to completed + add a comment" }
    fn hint(&self) -> &'static str {
        "Two commands:\n\
         1. `aida edit <FR-id> --status completed`\n\
         2. `aida comment add <FR-id> \"<short note about how it landed>\"`\n\n\
         The comment is what survives in the audit trail — leave context (sha, deferred work, etc.)."
    }
    // trace:STORY-20 | ai:claude
    fn hint_more(&self) -> Option<&'static str> {
        Some(
            "1. Run `aida edit FR-1 --status completed`.\n\
             2. Run `aida comment add FR-1 \"<note on how it landed>\"`.\n\
             3. `aida show FR-1 --comments` confirms both."
        )
    }
    // trace:STORY-20 | ai:claude
    fn hint_solution(&self) -> Option<&'static str> {
        Some(
            "aida edit FR-1 --status completed\n\
             aida comment add FR-1 \"Landed via the tutor walkthrough.\""
        )
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        if !is_aida_initialized(workspace) {
            return VerifyResult::Pending("complete exercise 01 first".into());
        }
        let frs = requirements_with_prefix(workspace, "FR");
        if frs.is_empty() {
            return VerifyResult::Pending("no FR-* present yet".into());
        }
        let any_completed = frs.iter().any(|r| {
            r.status
                .as_deref()
                .map(|s| s.eq_ignore_ascii_case("completed") || s.eq_ignore_ascii_case("Completed"))
                .unwrap_or(false)
        });
        if !any_completed {
            return VerifyResult::Pending(
                "no FR-* is completed yet — try `aida edit <FR-id> --status completed`".into()
            );
        }
        let any_with_comment = frs.iter().any(|r| r.comment_count > 0);
        if !any_with_comment {
            return VerifyResult::Pending(
                "FR-* is completed but has no comments — try `aida comment add <FR-id> \"...\"`".into()
            );
        }
        VerifyResult::Pass
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        let fr = demo_spec_id(workspace, "FR")?;
        run(workspace, "aida", &["edit", &fr, "--status", "completed"])?;
        run(workspace, "aida", &[
            "comment", "add", &fr, "Landed via the demo walkthrough.",
        ])
    }
}
