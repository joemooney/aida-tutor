//! Exercise 21 — enter a role: a persistent named hat that scopes your
//! queue and your prompt. trace:STORY-27 | ai:claude

use crate::exercise::{run, Exercise, VerifyResult};
use crate::verify::is_aida_initialized;
use std::path::Path;

pub struct E;

impl Exercise for E {
    fn id(&self) -> u32 { 21 }
    fn slug(&self) -> &'static str { "role-enter" }
    fn title(&self) -> &'static str { "aida role enter — wear a hat" }
    fn hint(&self) -> &'static str {
        "A role is a persistent named context — implementer, reviewer, dialog. `aida role list` \
         shows what's defined; `aida role enter implementer` switches to one. It prints shell \
         code you must `eval`, because a role attaches to your *shell session*: it exports \
         `AIDA_SESSION_ROLE` and prefixes your prompt. Run \
         `eval \"$(aida role enter implementer)\"`, then run `aida-tutor verify` in that same \
         terminal — the verifier reads the role from the shell it was launched in."
    }
    // trace:STORY-20 | ai:claude
    fn hint_more(&self) -> Option<&'static str> {
        Some(
            "1. Run `aida role scaffold` so the starter roles exist.\n\
             2. Run `eval \"$(aida role enter implementer)\"` — `enter` prints shell code you must `eval`.\n\
             3. Run `aida-tutor verify` in that *same* terminal."
        )
    }
    // trace:STORY-20 | ai:claude
    fn hint_solution(&self) -> Option<&'static str> {
        Some(
            "aida role scaffold\n\
             eval \"$(aida role enter implementer)\"\n\
             aida-tutor verify"
        )
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        if !is_aida_initialized(workspace) {
            return VerifyResult::Pending("complete exercise 01 first".into());
        }
        // A role lives in the shell session, not on disk: `aida role enter`
        // exports AIDA_SESSION_ROLE. `aida-tutor` inherits the env of the
        // shell that launched it, so an active role is visible to the
        // verifier ONLY when `verify` runs in the same terminal the role
        // was entered in — which is exactly the lesson of this exercise.
        // trace:STORY-27 | ai:claude
        match std::env::var("AIDA_SESSION_ROLE") {
            Ok(role) if !role.trim().is_empty() => VerifyResult::Pass,
            _ => VerifyResult::Pending(
                "no role active in this shell — run `eval \"$(aida role enter implementer)\"`, \
                 then run `aida-tutor verify` in that same terminal.".into(),
            ),
        }
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        // Make sure the starter roles exist (idempotent), enter one for
        // real so the demo exercises the live CLI surface, then mirror its
        // shell effect into this process's env so `verify` sees the role.
        run(workspace, "aida", &["role", "scaffold"])?;
        run(workspace, "aida", &["role", "enter", "implementer"])?;
        run(workspace, "aida", &["role", "list"])?;
        std::env::set_var("AIDA_SESSION_ROLE", "implementer");
        Ok(())
    }
}
