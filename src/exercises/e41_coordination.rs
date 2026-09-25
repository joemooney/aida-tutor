//! Exercise 41 — operator visibility and coordination. trace:STORY-60 | ai:codex

use crate::exercise::{run, verify_invocation, Exercise, VerifyResult};
use crate::verify::is_aida_initialized;
use std::path::Path;

pub struct E;

impl Exercise for E {
    fn id(&self) -> u32 {
        41
    }
    fn slug(&self) -> &'static str {
        "coordination"
    }
    fn title(&self) -> &'static str {
        "status / awaiting / health — see the work"
    }
    fn hint(&self) -> &'static str {
        "Use the operator loop instead of guessing: `aida status --no-ci`, `aida awaiting --no-ci`, `aida ps`, and `aida health --brief`. Add `aida mailbox list`, `aida team`, `aida node whoami`, and `aida worktree list` when coordinating more than one clone or agent."
    }
    fn hint_more(&self) -> Option<&'static str> {
        Some("1. Run the four read-only health/coordination views.\n2. Read what is live, stale, awaiting a human, or unhealthy.\n3. Inspect team identity and managed worktrees before assigning or launching work.")
    }
    fn hint_solution(&self) -> Option<&'static str> {
        Some("aida status --no-ci\naida awaiting --no-ci\naida ps\naida health --brief\naida mailbox list\naida team\naida node whoami\naida worktree list")
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        if !is_aida_initialized(workspace) {
            return VerifyResult::Pending("complete exercise 01 first".into());
        }
        for (command, args) in [
            ("status", vec!["--no-ci"]),
            ("awaiting", vec!["--no-ci"]),
            ("ps", vec![]),
            ("health", vec!["--brief"]),
        ] {
            let result = verify_invocation(
                workspace,
                command,
                &args,
                "run the coordination views from the hint",
            );
            if !matches!(result, VerifyResult::Pass) {
                return result;
            }
        }
        VerifyResult::Pass
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        run(workspace, "aida", &["status", "--no-ci"])?;
        run(workspace, "aida", &["awaiting", "--no-ci"])?;
        run(workspace, "aida", &["ps"])?;
        run(workspace, "aida", &["health", "--brief"])?;
        run(workspace, "aida", &["mailbox", "list"])?;
        run(workspace, "aida", &["team"])?;
        run(workspace, "aida", &["node", "whoami"])?;
        run(workspace, "aida", &["worktree", "list"])
    }
}
