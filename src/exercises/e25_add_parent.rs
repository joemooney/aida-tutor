//! Exercise 25 — capture a requirement under a parent with
//! `aida add --parent`. Cluster 2 (relationships). trace:STORY-26 | ai:claude

use crate::exercise::{demo_req_by_title, run, Exercise, VerifyResult};
use crate::verify::{is_aida_initialized, requirement_by_title};
use std::path::Path;

pub struct E;

/// Titles for the umbrella epic and its child story. The content tells
/// the learner to use them verbatim; the verifier and demo follow each by
/// a unique case-insensitive substring. trace:STORY-26 | ai:claude
const EPIC_TITLE: &str = "Relationships demo: umbrella epic";
const EPIC_NEEDLE: &str = "umbrella epic";
const CHILD_TITLE: &str = "Relationships demo: child story";
const CHILD_NEEDLE: &str = "child story";

impl Exercise for E {
    fn id(&self) -> u32 { 25 }
    fn slug(&self) -> &'static str { "add-parent" }
    fn title(&self) -> &'static str { "aida add --parent — capture under a parent" }
    fn hint(&self) -> &'static str {
        "Requirements form a graph, not a flat list. `aida add --parent <ID>` captures a new \
         requirement AND links it under `<ID>` in one step. AIDA writes the edge on both \
         endpoints — the child gets a `Child` edge to the parent, the parent gets the inverse \
         `Parent` edge back. First capture an epic (`aida add --type epic --status approved \
         --title \"Relationships demo: umbrella epic\"`), note the ID it prints, then capture \
         a story with `--parent <that-ID> --type story`."
    }
    // trace:STORY-20 | ai:claude
    fn hint_more(&self) -> Option<&'static str> {
        Some(
            "1. Capture an epic: `aida add --type epic --status approved \
                --title \"Relationships demo: umbrella epic\"`.\n\
             2. Note the id it prints.\n\
             3. Capture a story under it: `aida add --parent <epic-id> --type story \
                --status approved --title \"Relationships demo: child story\"`."
        )
    }
    // trace:STORY-20 | ai:claude
    fn hint_solution(&self) -> Option<&'static str> {
        Some(
            "aida add --type epic --status approved --title \"Relationships demo: umbrella epic\"\n\
             aida add --parent <EPIC-ID> --type story --status approved --title \"Relationships demo: child story\""
        )
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        if !is_aida_initialized(workspace) {
            return VerifyResult::Pending("complete exercise 01 first".into());
        }
        let Some(epic) = requirement_by_title(workspace, EPIC_NEEDLE) else {
            return VerifyResult::Pending(
                "no \"umbrella epic\" captured yet — run `aida add --type epic --status \
                 approved --title \"Relationships demo: umbrella epic\"`.".into(),
            );
        };
        let Some(child) = requirement_by_title(workspace, CHILD_NEEDLE) else {
            return VerifyResult::Pending(
                "the epic exists, but no \"child story\" yet — capture one under it with \
                 `aida add --parent <EPIC-ID> --type story --status approved --title \
                 \"Relationships demo: child story\"`.".into(),
            );
        };
        let epic_uuid = epic.uuid.as_deref().unwrap_or_default();
        let child_uuid = child.uuid.as_deref().unwrap_or_default();
        // The child must carry a `Child` edge pointing at the epic.
        if !child.has_edge("Child", epic_uuid) {
            return VerifyResult::Pending(format!(
                "\"{}\" exists but isn't linked under the epic — capture it with \
                 `aida add --parent {} ...`, or link it after the fact with \
                 `aida rel add {} {} --type child --bidirectional`.",
                child.title.as_deref().unwrap_or("the story"),
                epic.spec_id.as_deref().unwrap_or("<EPIC-ID>"),
                child.spec_id.as_deref().unwrap_or("<STORY-ID>"),
                epic.spec_id.as_deref().unwrap_or("<EPIC-ID>"),
            ));
        }
        // ...and the epic must carry the inverse `Parent` edge back. AIDA
        // writes both sides; a missing inverse means a half-written graph.
        if !epic.has_edge("Parent", child_uuid) {
            return VerifyResult::Fail(
                "the story points at the epic, but the epic has no inverse `Parent` edge \
                 back — the relationship is only half-written.".into(),
            );
        }
        VerifyResult::Pass
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        // Capture the umbrella epic, resolve its creation-order-dependent
        // spec_id, then capture a child story under it with `--parent`.
        run(workspace, "aida", &[
            "add", "--type", "epic", "--status", "approved", "--title", EPIC_TITLE,
        ])?;
        let epic_id = demo_req_by_title(workspace, EPIC_NEEDLE)?;
        run(workspace, "aida", &[
            "add", "--parent", &epic_id, "--type", "story", "--status", "approved",
            "--title", CHILD_TITLE,
        ])
    }
}
