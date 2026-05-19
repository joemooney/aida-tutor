//! Exercise 24 — finish a queued item: `aida queue done` completes the
//! requirement AND dequeues it in one atomic step. trace:STORY-27 | ai:claude

use super::e22_queue_add::QUEUE_DEMO_NEEDLE;
use crate::exercise::{demo_req_by_title, run, Exercise, VerifyResult};
use crate::verify::{is_aida_initialized, queue_entries, requirement_by_title};
use std::path::Path;

pub struct E;

impl Exercise for E {
    fn id(&self) -> u32 { 24 }
    fn slug(&self) -> &'static str { "queue-done" }
    fn title(&self) -> &'static str { "aida queue done — finish + dequeue atomically" }
    fn hint(&self) -> &'static str {
        "`aida queue done <ID>` is the consumer's closing move: it marks the requirement done \
         AND removes it from the queue in one step — no chance of a finished item lingering in \
         the queue, or a dequeued item left un-closed. It's equivalent to `aida edit <ID> \
         --status completed` plus `aida queue remove <ID>`, but atomic. Pass `--yes` to skip \
         the confirmation prompt."
    }
    // trace:STORY-20 | ai:claude
    fn hint_more(&self) -> Option<&'static str> {
        Some(
            "1. The \"Queue demo\" task is in-progress from exercise 23.\n\
             2. Run `aida queue done <id>` — it completes the requirement AND dequeues it atomically.\n\
             3. Add `--yes` to skip the confirmation prompt."
        )
    }
    // trace:STORY-20 | ai:claude
    fn hint_solution(&self) -> Option<&'static str> {
        Some("aida queue done <ID> --yes                 # <ID> = the \"Queue demo\" task")
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
        let finished = task
            .status
            .as_deref()
            .map(|s| {
                let s = s.to_lowercase();
                s == "done" || s == "completed"
            })
            .unwrap_or(false);
        if !finished {
            return VerifyResult::Pending(format!(
                "\"Queue demo\" task isn't finished yet — run `aida queue done {} --yes`.",
                task.spec_id.as_deref().unwrap_or("<ID>"),
            ));
        }
        // `queue done` is atomic — a finished task must no longer be in
        // any queue. If it is, the dequeue half didn't happen.
        let uuid = task.uuid.as_deref().unwrap_or_default();
        let still_queued = queue_entries(workspace)
            .into_iter()
            .any(|e| e.requirement_id.as_deref() == Some(uuid));
        if still_queued {
            return VerifyResult::Fail(
                "the \"Queue demo\" task is done but still sitting in the queue — \
                 `aida queue done` should have removed it. Try `aida queue remove <ID>`.".into(),
            );
        }
        VerifyResult::Pass
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        // Closing move: complete + dequeue the cluster's task in one step.
        let id = demo_req_by_title(workspace, QUEUE_DEMO_NEEDLE)?;
        run(workspace, "aida", &["queue", "done", &id, "--yes"])
    }
}
