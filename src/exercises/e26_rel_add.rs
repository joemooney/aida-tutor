//! Exercise 26 — link two existing requirements with a typed relationship
//! via `aida rel add`. Cluster 2 (relationships). trace:STORY-26 | ai:claude

use crate::exercise::{demo_req_by_title, run, Exercise, VerifyResult};
use crate::verify::{is_aida_initialized, requirement_by_title};
use std::path::Path;

pub struct E;

/// Titles for the feature and the task that verifies it. The content
/// tells the learner to use them verbatim; the verifier and demo follow
/// each by a unique case-insensitive substring. trace:STORY-26 | ai:claude
const FEATURE_TITLE: &str = "Relationships demo: feature under test";
const FEATURE_NEEDLE: &str = "feature under test";
const TASK_TITLE: &str = "Relationships demo: verifying task";
const TASK_NEEDLE: &str = "verifying task";

impl Exercise for E {
    fn id(&self) -> u32 { 26 }
    fn slug(&self) -> &'static str { "rel-add" }
    fn title(&self) -> &'static str { "aida rel add — typed relationships" }
    fn hint(&self) -> &'static str {
        "`aida rel add <FROM> <TO> --type <kind>` links two requirements that already exist. \
         Use `--type verifies` for a task that proves a feature works. By default only the \
         source gets the edge; pass `--bidirectional` and AIDA also writes the inverse edge \
         (`VerifiedBy`) on the target. Capture a feature and a task, note both IDs, then run \
         `aida rel add <TASK-ID> <FR-ID> --type verifies --bidirectional`."
    }
    // trace:STORY-20 | ai:claude
    fn hint_more(&self) -> Option<&'static str> {
        Some(
            "1. Capture a feature titled \"...feature under test\" and a task titled \"...verifying task\".\n\
             2. Note both ids.\n\
             3. Link them: `aida rel add <task-id> <feature-id> --type verifies --bidirectional`."
        )
    }
    // trace:STORY-20 | ai:claude
    fn hint_solution(&self) -> Option<&'static str> {
        Some(
            "aida add --type functional --status approved --title \"Relationships demo: feature under test\"\n\
             aida add --type task --status approved --title \"Relationships demo: verifying task\"\n\
             aida rel add <TASK-ID> <FR-ID> --type verifies --bidirectional"
        )
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        if !is_aida_initialized(workspace) {
            return VerifyResult::Pending("complete exercise 01 first".into());
        }
        let Some(feature) = requirement_by_title(workspace, FEATURE_NEEDLE) else {
            return VerifyResult::Pending(
                "no \"feature under test\" captured yet — run `aida add --type functional \
                 --status approved --title \"Relationships demo: feature under test\"`.".into(),
            );
        };
        let Some(task) = requirement_by_title(workspace, TASK_NEEDLE) else {
            return VerifyResult::Pending(
                "the feature exists, but no \"verifying task\" yet — run `aida add --type \
                 task --status approved --title \"Relationships demo: verifying task\"`.".into(),
            );
        };
        let feature_uuid = feature.uuid.as_deref().unwrap_or_default();
        let task_uuid = task.uuid.as_deref().unwrap_or_default();
        // The task must carry a `Verifies` edge to the feature.
        if !task.has_edge("Verifies", feature_uuid) {
            return VerifyResult::Pending(format!(
                "the feature and task both exist but aren't linked yet — run \
                 `aida rel add {} {} --type verifies --bidirectional`.",
                task.spec_id.as_deref().unwrap_or("<TASK-ID>"),
                feature.spec_id.as_deref().unwrap_or("<FR-ID>"),
            ));
        }
        // ...and `--bidirectional` must have written the inverse on the
        // feature: a `VerifiedBy` edge pointing back at the task.
        if !feature.has_edge("VerifiedBy", task_uuid) {
            return VerifyResult::Pending(format!(
                "the task verifies the feature, but the feature has no inverse `VerifiedBy` \
                 edge — you left off `--bidirectional`. Re-run with the flag, or add the \
                 inverse directly: `aida rel add {} {} --type verified-by`.",
                feature.spec_id.as_deref().unwrap_or("<FR-ID>"),
                task.spec_id.as_deref().unwrap_or("<TASK-ID>"),
            ));
        }
        VerifyResult::Pass
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        // Capture the feature + the verifying task, resolve their
        // creation-order-dependent spec_ids, then link them bidirectionally.
        run(workspace, "aida", &[
            "add", "--type", "functional", "--status", "approved", "--title", FEATURE_TITLE,
        ])?;
        run(workspace, "aida", &[
            "add", "--type", "task", "--status", "approved", "--title", TASK_TITLE,
        ])?;
        let feature_id = demo_req_by_title(workspace, FEATURE_NEEDLE)?;
        let task_id = demo_req_by_title(workspace, TASK_NEEDLE)?;
        run(workspace, "aida", &[
            "rel", "add", &task_id, &feature_id, "--type", "verifies", "--bidirectional",
        ])
    }
}
