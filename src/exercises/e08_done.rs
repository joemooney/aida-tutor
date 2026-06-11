//! Exercise 08 — close the loop: `aida done FR-1`. trace:STORY-46 | ai:claude

use crate::exercise::{demo_spec_id, run, Exercise, VerifyResult};
use crate::verify::{is_aida_initialized, requirements_with_prefix};
use std::path::Path;

pub struct E;

impl Exercise for E {
    fn id(&self) -> u32 {
        8
    }
    fn slug(&self) -> &'static str {
        "done"
    }
    fn title(&self) -> &'static str {
        "aida done FR-1 — mark it finished"
    }
    fn hint(&self) -> &'static str {
        "One command: `aida done FR-1`. That's the newcomer-friendly way to say \"I finished \
         it\" — it flips the status to completed. (Under the hood it's the same as \
         `aida edit FR-1 --status completed`; `done` is just the short, no-jargon form.) \
         Optionally, leave a note for the audit trail with `aida comment add FR-1 \"...\"`."
    }
    // trace:STORY-20 | ai:claude
    fn hint_more(&self) -> Option<&'static str> {
        Some(
            "1. Run `aida done FR-1`.\n\
             2. `aida show FR-1` confirms the status is now Completed.\n\
             3. Optional: `aida comment add FR-1 \"<how it landed>\"` for the audit trail.",
        )
    }
    // trace:STORY-20 | ai:claude
    fn hint_solution(&self) -> Option<&'static str> {
        Some("aida done FR-1")
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        if !is_aida_initialized(workspace) {
            return VerifyResult::Pending("complete exercise 01 first".into());
        }
        let frs = requirements_with_prefix(workspace, "FR");
        if frs.is_empty() {
            return VerifyResult::Pending("no FR-* present yet".into());
        }
        // `aida done` sets status Completed; accept the literal "done" too
        // in case a learner reaches for `edit --status done`.
        let any_closed = frs.iter().any(|r| {
            r.status
                .as_deref()
                .map(|s| s.eq_ignore_ascii_case("completed") || s.eq_ignore_ascii_case("done"))
                .unwrap_or(false)
        });
        if !any_closed {
            return VerifyResult::Pending(
                "FR-1 isn't marked finished yet — run `aida done FR-1`".into(),
            );
        }
        VerifyResult::Pass
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        let fr = demo_spec_id(workspace, "FR")?;
        run(workspace, "aida", &["done", &fr])?;
        run(
            workspace,
            "aida",
            &["comment", "add", &fr, "Landed via the demo walkthrough."],
        )
    }
}
