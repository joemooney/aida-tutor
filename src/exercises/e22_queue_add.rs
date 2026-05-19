//! Exercise 22 — route work to a role's queue: the producer side of the
//! producer/consumer loop. trace:STORY-27 | ai:claude

use crate::exercise::{demo_req_by_title, run, Exercise, VerifyResult};
use crate::verify::{is_aida_initialized, queue_entries, requirement_by_title};
use std::path::Path;

pub struct E;

/// The exact title the queue-cluster task carries. Exercises 22-24 follow
/// this one requirement across capture → pickup → done; the content tells
/// the learner to use this title verbatim. trace:STORY-27 | ai:claude
pub const QUEUE_DEMO_TITLE: &str = "Queue demo: ship the sample widget";
/// Case-insensitive substring the verifiers match the title on.
pub const QUEUE_DEMO_NEEDLE: &str = "queue demo";

impl Exercise for E {
    fn id(&self) -> u32 { 22 }
    fn slug(&self) -> &'static str { "queue-add" }
    fn title(&self) -> &'static str { "aida queue add --for — route work to a role" }
    fn hint(&self) -> &'static str {
        "The queue is a per-role worklist. Whoever wears the `dialog` hat routes work to a doer \
         role with `aida queue add <ID> --for implementer`. First capture the task this cluster \
         follows — `aida add --type task --status approved --title \"Queue demo: ship the \
         sample widget\"` — note the ID it prints, then queue that ID with `--for implementer`."
    }
    // trace:STORY-20 | ai:claude
    fn hint_more(&self) -> Option<&'static str> {
        Some(
            "1. Capture the cluster's task: `aida add --type task --status approved \
                --title \"Queue demo: ship the sample widget\"`.\n\
             2. Note the id it prints.\n\
             3. Route it to a doer role: `aida queue add <that-id> --for implementer`."
        )
    }
    // trace:STORY-20 | ai:claude
    fn hint_solution(&self) -> Option<&'static str> {
        Some(
            "aida add --type task --status approved --title \"Queue demo: ship the sample widget\"\n\
             aida queue add <ID> --for implementer       # <ID> = the id the add above printed"
        )
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        if !is_aida_initialized(workspace) {
            return VerifyResult::Pending("complete exercise 01 first".into());
        }
        let Some(task) = requirement_by_title(workspace, QUEUE_DEMO_NEEDLE) else {
            return VerifyResult::Pending(
                "no \"Queue demo\" task captured yet — run `aida add --type task --status \
                 approved --title \"Queue demo: ship the sample widget\"`.".into(),
            );
        };
        let uuid = task.uuid.as_deref().unwrap_or_default();
        let routed = queue_entries(workspace).into_iter().any(|e| {
            e.requirement_id.as_deref() == Some(uuid)
                && e.for_role
                    .as_deref()
                    .map(|r| !r.trim().is_empty())
                    .unwrap_or(false)
        });
        if !routed {
            return VerifyResult::Pending(format!(
                "\"{}\" exists but isn't routed to a role's queue yet — run \
                 `aida queue add {} --for implementer`.",
                task.title.as_deref().unwrap_or("the task"),
                task.spec_id.as_deref().unwrap_or("<ID>"),
            ));
        }
        VerifyResult::Pass
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        // Producer side: capture the cluster's task, then route it to the
        // implementer queue. The spec_id depends on creation order, so we
        // resolve it back from the title rather than hardcoding it.
        run(workspace, "aida", &[
            "add", "--type", "task", "--status", "approved",
            "--title", QUEUE_DEMO_TITLE,
        ])?;
        let id = demo_req_by_title(workspace, QUEUE_DEMO_NEEDLE)?;
        run(workspace, "aida", &["queue", "add", &id, "--for", "implementer"])
    }
}
