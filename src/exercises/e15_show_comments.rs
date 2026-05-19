//! Exercise 15 — `aida show <id> --comments`. trace:STORY-15 | ai:claude

use crate::exercise::{demo_spec_id, run, verify_invocation, Exercise, VerifyResult};
use crate::verify::is_aida_initialized;
use std::path::Path;

pub struct E;

impl Exercise for E {
    fn id(&self) -> u32 { 15 }
    fn slug(&self) -> &'static str { "show-comments" }
    fn title(&self) -> &'static str { "aida show <id> --comments — read the audit trail" }
    fn hint(&self) -> &'static str {
        "Run `aida show <FR-id> --comments`. You'll see the full record + every comment with author + \
         timestamp + content. This is what survives in the audit log when teammates ask 'when did we \
         decide that?' six months later."
    }
    // trace:STORY-20 | ai:claude
    fn hint_more(&self) -> Option<&'static str> {
        Some(
            "1. FR-1 picked up a comment in exercise 14.\n\
             2. Run `aida show FR-1` with the `--comments` flag.\n\
             3. Each comment shows author, timestamp and body."
        )
    }
    // trace:STORY-20 | ai:claude
    fn hint_solution(&self) -> Option<&'static str> {
        Some("aida show FR-1 --comments")
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        if !is_aida_initialized(workspace) {
            return VerifyResult::Pending("complete exercise 01 first".into());
        }
        // Read-only command — see exercise 07. The wrapper opt-in here
        // also checks the `--comments` flag actually appeared, not just
        // a bare `aida show`. trace:STORY-22 | ai:claude
        verify_invocation(
            workspace,
            "show",
            &["--comments"],
            "the invocation wrapper shows no `aida show <id> --comments` yet — run it with --comments",
        )
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        let fr = demo_spec_id(workspace, "FR")?;
        run(workspace, "aida", &["show", &fr, "--comments"])
    }
}
