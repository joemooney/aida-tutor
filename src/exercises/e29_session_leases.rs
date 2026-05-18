//! Exercise 29 — see who holds what: `aida session leases` lists the
//! active leases, `aida session show <id>` drills into one. Cluster 4
//! (sessions + worktrees). trace:STORY-28 | ai:claude

use super::e27_session_start::SESSION_BRANCH;
use crate::exercise::{demo_session_lease, run, Exercise, VerifyResult};
use crate::verify::{is_aida_initialized, session_lease_for_branch};
use std::path::Path;

pub struct E;

impl Exercise for E {
    fn id(&self) -> u32 { 29 }
    fn slug(&self) -> &'static str { "session-leases" }
    fn title(&self) -> &'static str { "aida session leases — who holds what" }
    fn hint(&self) -> &'static str {
        "When several sessions run at once, the lease list is how they coordinate. `aida \
         session leases` prints one row per active session — id, scope, branch, role, \
         worktree — the canonical \"who holds what right now\" view. `aida session show <id>` \
         drills into one: its worktree path, the lease file, recent activity, whether a live \
         `claude` is inside. Run `aida session leases` to find your session's id, then \
         `aida session show <id>` on it. Both are read-only — they inspect, they don't change \
         anything."
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        if !is_aida_initialized(workspace) {
            return VerifyResult::Pending("complete exercise 01 first".into());
        }
        // `session leases` / `session show` are read-only — there's no
        // on-disk trace of having run them. Passing on the prerequisite
        // (an active session lease exists to inspect) matches the other
        // read-only exercises in the tutor. trace:STORY-28 | ai:claude
        if session_lease_for_branch(workspace, SESSION_BRANCH).is_some() {
            VerifyResult::Pass
        } else {
            VerifyResult::Pending(
                "no active session lease to inspect — complete exercise 27 first, and don't \
                 end the session until after this exercise.".into(),
            )
        }
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        // Inspect the live session: list every lease, then drill into the
        // one this cluster created.
        run(workspace, "aida", &["session", "leases"])?;
        let lease = demo_session_lease(workspace, SESSION_BRANCH)?;
        let id = lease
            .id
            .ok_or_else(|| anyhow::anyhow!("demo: session lease has no id"))?;
        run(workspace, "aida", &["session", "show", &id])
    }
}
