//! Exercise 38 — graph traversal and focus. trace:STORY-57 | ai:codex

use crate::exercise::{demo_spec_id, run, Exercise, VerifyResult};
use crate::verify::is_aida_initialized;
use std::path::Path;

pub struct E;

impl Exercise for E {
    fn id(&self) -> u32 {
        38
    }
    fn slug(&self) -> &'static str {
        "graph-focus"
    }
    fn title(&self) -> &'static str {
        "graph + focus — narrow the context"
    }
    fn hint(&self) -> &'static str {
        "Use `aida graph FR-1` to see the requirement neighborhood, then `aida focus FR-1` to make list/status reads stay inside that context. `aida focus --clear` removes the namespace when you are done."
    }
    fn hint_more(&self) -> Option<&'static str> {
        Some("1. Run `aida graph FR-1 --tree` (or use the feature id you captured).\n2. Set the focus to that id.\n3. Run `aida focus` and `aida list`; clear it when finished.")
    }
    fn hint_solution(&self) -> Option<&'static str> {
        Some("aida graph FR-1 --tree\naida focus FR-1\naida focus\naida focus --clear")
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        if !is_aida_initialized(workspace) {
            return VerifyResult::Pending("complete exercise 01 first".into());
        }
        let Some(fr) = demo_spec_id(workspace, "FR").ok() else {
            return VerifyResult::Pending("capture a functional requirement first".into());
        };
        let focus = workspace.join(".aida").join("focus");
        match std::fs::read_to_string(focus) {
            Ok(value) if value.trim() == fr => VerifyResult::Pass,
            Ok(value) => {
                VerifyResult::Fail(format!("focus is `{}`; expected `{fr}`", value.trim()))
            }
            Err(_) => VerifyResult::Pending(format!("set the focus with `aida focus {fr}`")),
        }
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        run(workspace, "aida", &["focus", "--clear"])?;
        let fr = demo_spec_id(workspace, "FR")?;
        run(workspace, "aida", &["graph", &fr, "--tree"])?;
        run(workspace, "aida", &["focus", &fr])
    }
}
