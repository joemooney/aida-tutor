//! Exercise 23 — pick up a queued item: peek the head, claim it by
//! flipping it in-progress. The consumer side. trace:STORY-27 | ai:claude

use super::e22_queue_add::QUEUE_DEMO_NEEDLE;
use crate::exercise::{demo_req_by_title, run, Exercise, VerifyResult};
use crate::verify::{is_aida_initialized, requirement_by_title};
use std::path::Path;

pub struct E;

impl Exercise for E {
    fn id(&self) -> u32 { 23 }
    fn slug(&self) -> &'static str { "queue-pickup" }
    fn title(&self) -> &'static str { "aida queue next — pick up the next item" }
    fn hint(&self) -> &'static str {
        "`aida queue next` peeks at the top item routed to your active role without removing it. \
         To claim it, flip its status: `aida edit <ID> --status in-progress`. Inside Claude Code \
         the `/aida-pickup` slash command does exactly this — peek the head, mark it in-progress, \
         render the spec — but the moving parts are just `queue next` + `edit`."
    }
    // trace:STORY-20 | ai:claude
    fn hint_more(&self) -> Option<&'static str> {
        Some(
            "1. Run `aida queue next` — it peeks at the head item routed to your role.\n\
             2. Note that item's id.\n\
             3. Claim it: `aida edit <id> --status in-progress`."
        )
    }
    // trace:STORY-20 | ai:claude
    fn hint_solution(&self) -> Option<&'static str> {
        Some(
            "aida queue next\n\
             aida edit <ID> --status in-progress         # <ID> = the \"Queue demo\" task"
        )
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        if !is_aida_initialized(workspace) {
            return VerifyResult::Pending("complete exercise 01 first".into());
        }
        let Some(task) = requirement_by_title(workspace, QUEUE_DEMO_NEEDLE) else {
            return VerifyResult::Pending(
                "no \"Queue demo\" task in the store — complete exercise 22 first.".into(),
            );
        };
        let in_progress = task
            .status
            .as_deref()
            .map(|s| {
                let s = s.to_lowercase();
                s == "in-progress" || s == "inprogress" || s == "in progress"
            })
            .unwrap_or(false);
        if !in_progress {
            return VerifyResult::Pending(format!(
                "\"Queue demo\" task isn't claimed yet — run `aida queue next` to see it, \
                 then `aida edit {} --status in-progress` to pick it up.",
                task.spec_id.as_deref().unwrap_or("<ID>"),
            ));
        }
        VerifyResult::Pass
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        // Consumer side: peek the queue head, then claim the cluster's
        // task by flipping it in-progress. `queue next` filters by the
        // active role, set in exercise 21's demo.
        run(workspace, "aida", &["queue", "next"])?;
        let id = demo_req_by_title(workspace, QUEUE_DEMO_NEEDLE)?;
        run(workspace, "aida", &["edit", &id, "--status", "in-progress"])
    }
}
