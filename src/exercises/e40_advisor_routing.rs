//! Exercise 40 — advisor routing and safe punts. trace:STORY-59 | ai:codex

use crate::exercise::{demo_req_by_title, run, Exercise, VerifyResult};
use crate::verify::{is_aida_initialized, requirement_by_title};
use std::path::Path;

pub struct E;

impl Exercise for E {
    fn id(&self) -> u32 {
        40
    }
    fn slug(&self) -> &'static str {
        "advisor-routing"
    }
    fn title(&self) -> &'static str {
        "groom / questions / punt — stop and route uncertainty"
    }
    fn hint(&self) -> &'static str {
        "When a requirement is ambiguous, do not guess. `aida punt` moves an in-progress spec to Needs Attention with a structured reason; `aida questions list` shows the decision inbox; `aida groom --dry-run` previews advisor intake without approving anything."
    }
    fn hint_more(&self) -> Option<&'static str> {
        Some("1. Create or select an in-progress task with a real design fork.\n2. Punt it with category `design-fork` and a reason.\n3. Read `aida questions list`; use `aida groom --dry-run` to inspect intake without applying it.")
    }
    fn hint_solution(&self) -> Option<&'static str> {
        Some("aida punt TASK-ID --category design-fork --reason 'Two valid designs need a human choice.'\naida questions list\naida groom --dry-run")
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        if !is_aida_initialized(workspace) {
            return VerifyResult::Pending("complete exercise 01 first".into());
        }
        let Some(req) = requirement_by_title(workspace, "Tutor routing demo") else {
            return VerifyResult::Pending("create the Tutor routing demo task and punt it".into());
        };
        match req
            .status
            .as_deref()
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str()
        {
            "needs-attention" | "needs attention" | "needs_attention" | "needsattention" => {
                VerifyResult::Pass
            }
            status => VerifyResult::Pending(format!(
                "Tutor routing demo is `{status}`; punt it with a design-fork reason"
            )),
        }
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        // Filing and status transitions are advisor actions; punt then
        // hands the unresolved fork to the decision inbox. trace:STORY-59
        std::env::set_var("AIDA_SESSION_ROLE", "advisor");
        run(
            workspace,
            "aida",
            &[
                "add",
                "--type",
                "task",
                "--status",
                "in-progress",
                "--priority",
                "low",
                "--title",
                "Tutor routing demo",
            ],
        )?;
        let id = demo_req_by_title(workspace, "Tutor routing demo")?;
        run(
            workspace,
            "aida",
            &[
                "punt",
                &id,
                "--category",
                "design-fork",
                "--reason",
                "The learner must choose between two valid designs.",
            ],
        )?;
        run(workspace, "aida", &["questions", "list"])?;
        run(workspace, "aida", &["groom", "--dry-run"])
    }
}
