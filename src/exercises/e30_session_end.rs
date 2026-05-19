//! Exercise 30 — release a session: `aida session end` removes the
//! worktree and deletes the lease, but keeps the branch and its commits.
//! Cluster 4 (sessions + worktrees). trace:STORY-28 | ai:claude

use super::e27_session_start::SESSION_BRANCH;
use crate::exercise::{demo_session_lease, run, Exercise, VerifyResult};
use crate::verify::{git_branch_exists, git_worktrees, is_aida_initialized, session_lease_for_branch};
use std::path::Path;

pub struct E;

impl Exercise for E {
    fn id(&self) -> u32 { 30 }
    fn slug(&self) -> &'static str { "session-end" }
    fn title(&self) -> &'static str { "aida session end — release the worktree" }
    fn hint(&self) -> &'static str {
        "`aida session end <id>` closes a session: it removes the git worktree and deletes the \
         lease — but deliberately leaves the branch alone, so the commits you made in exercise \
         28 are safe to merge or discard on your own schedule. Pass `--yes` to skip the \
         confirmation. Find the id with `aida session leases` (or `aida session show`), then \
         end it. Afterward `git worktree list` no longer shows the worktree, `aida session \
         leases` is empty — but `git branch` still lists `session-work`."
    }
    // trace:STORY-20 | ai:claude
    fn hint_more(&self) -> Option<&'static str> {
        Some(
            "1. Run `aida session leases` to find your open session's id.\n\
             2. Run `aida session end <id>` — pass `--yes` to skip the confirmation.\n\
             3. The worktree and lease are gone afterward; the `session-work` branch stays."
        )
    }
    // trace:STORY-20 | ai:claude
    fn hint_solution(&self) -> Option<&'static str> {
        Some("aida session end <SESSION-ID> --yes")
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        if !is_aida_initialized(workspace) {
            return VerifyResult::Pending("complete exercise 01 first".into());
        }
        // The session branch outlives the session — `end` keeps it — so
        // its presence is the proof a session was ever started here. No
        // branch means exercise 27 hasn't been done.
        if !git_branch_exists(workspace, SESSION_BRANCH) {
            return VerifyResult::Pending(
                "no `session-work` branch — complete exercise 27 first.".into(),
            );
        }
        let lease = session_lease_for_branch(workspace, SESSION_BRANCH);
        let worktree_remains = git_worktrees(workspace)
            .iter()
            .any(|w| w.branch.as_deref() == Some(SESSION_BRANCH));
        if lease.is_some() || worktree_remains {
            let id = lease.and_then(|l| l.id).unwrap_or_else(|| "<id>".into());
            return VerifyResult::Pending(format!(
                "the session is still open — run `aida session end {id} --yes` to remove the \
                 worktree and release the lease (the branch stays)."
            ));
        }
        VerifyResult::Pass
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        // Closing move: end the session by id. `--skip-ci` keeps the demo
        // offline (no `gh` probe); the worktree is clean because exercise
        // 28 committed its work, so no `--force` is needed.
        let lease = demo_session_lease(workspace, SESSION_BRANCH)?;
        let id = lease
            .id
            .ok_or_else(|| anyhow::anyhow!("demo: session lease has no id"))?;
        run(workspace, "aida", &["session", "end", &id, "--yes", "--skip-ci"])
    }
}
