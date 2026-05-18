//! Exercise 08 — `aida show <id>`. trace:STORY-8 | ai:claude

use crate::exercise::{demo_spec_id, run, Exercise, VerifyResult};
use crate::verify::is_aida_initialized;
use std::path::Path;

pub struct E;

impl Exercise for E {
    fn id(&self) -> u32 { 8 }
    fn slug(&self) -> &'static str { "show" }
    fn title(&self) -> &'static str { "aida show <id> — read a single requirement" }
    fn hint(&self) -> &'static str {
        "Run `aida show <id>` for any of your created reqs (e.g. `aida show FR-1`). \
         You'll see the title, status, priority, and the full description as readable prose. \
         If you also pass `--comments`, any audit-trail comments appear at the bottom."
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        if !is_aida_initialized(workspace) {
            return VerifyResult::Pending("complete exercise 01 first".into());
        }
        VerifyResult::Pass
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        let fr = demo_spec_id(workspace, "FR")?;
        run(workspace, "aida", &["show", &fr])
    }
}
