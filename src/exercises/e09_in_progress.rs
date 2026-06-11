//! Exercise 09 — reopen FR-1 to in-progress (the status vocabulary, and
//! the `--force` guard on closed reqs). trace:STORY-9 | ai:claude

use crate::exercise::{demo_spec_id, run, Exercise, VerifyResult};
use crate::verify::{is_aida_initialized, requirements_with_prefix};
use std::path::Path;

pub struct E;

impl Exercise for E {
    fn id(&self) -> u32 {
        9
    }
    fn slug(&self) -> &'static str {
        "in-progress"
    }
    fn title(&self) -> &'static str {
        "reopen FR-1 to in-progress (and meet --force)"
    }
    fn hint(&self) -> &'static str {
        "You marked FR-1 done in exercise 08 — but say a follow-up lands and you need to pick it \
         back up. `aida edit FR-1 --status in-progress` is the 'actively working' signal. AIDA \
         guards reopening a closed req, so it'll ask for `--force`: \
         `aida edit FR-1 --status in-progress --force`. (Inside Claude Code, `/aida-pickup` flips \
         status for you off the work queue — but the moving part is just `aida edit --status`.)"
    }
    // trace:STORY-20 | ai:claude
    fn hint_more(&self) -> Option<&'static str> {
        Some(
            "1. FR-1 is currently Completed (you closed it in exercise 08).\n\
             2. Reopening a closed req needs `--force`: `aida edit FR-1 --status in-progress --force`.\n\
             3. `aida show FR-1` confirms it's back In Progress.",
        )
    }
    // trace:STORY-20 | ai:claude
    fn hint_solution(&self) -> Option<&'static str> {
        Some("aida edit FR-1 --status in-progress --force")
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        if !is_aida_initialized(workspace) {
            return VerifyResult::Pending("complete exercise 01 first".into());
        }
        let frs = requirements_with_prefix(workspace, "FR");
        if frs.is_empty() {
            return VerifyResult::Pending("no FR-* req present yet (exercise 02)".into());
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
                "no FR-* is in-progress yet — reopen it with \
                 `aida edit FR-1 --status in-progress --force`"
                    .into(),
            );
        }
        VerifyResult::Pass
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        // FR-1 was closed in exercise 08; reopening a completed req needs
        // --force (AIDA guards accidental reopens). trace:STORY-46
        let fr = demo_spec_id(workspace, "FR")?;
        run(
            workspace,
            "aida",
            &["edit", &fr, "--status", "in-progress", "--force"],
        )
    }
}
