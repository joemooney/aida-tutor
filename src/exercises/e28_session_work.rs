//! Exercise 28 — work in the session worktree: commit on the session
//! branch, leaving the parent's branch untouched. Cluster 4 (sessions +
//! worktrees). trace:STORY-28 | ai:claude

use super::e27_session_start::SESSION_BRANCH;
use crate::exercise::{demo_session_lease, run, Exercise, VerifyResult};
use crate::verify::{
    git_branch_exists, git_commits_ahead, is_aida_initialized, main_worktree_branch,
};
use std::path::Path;

pub struct E;

impl Exercise for E {
    fn id(&self) -> u32 { 28 }
    fn slug(&self) -> &'static str { "session-work" }
    fn title(&self) -> &'static str { "work in the worktree — an isolated branch" }
    fn hint(&self) -> &'static str {
        "The session worktree is a real, separate checkout — `cd` into it (the sibling \
         directory `aida session start` printed) and work there. Anything you commit lands on \
         the `session-work` branch; the parent's branch never moves, so the two can't step on \
         each other. The AIDA store, by contrast, is symlinked in — requirement edits stay \
         shared. Make a change in the worktree (a file, a trace comment) and `git commit` it; \
         this exercise passes once the `session-work` branch has a commit the base branch \
         doesn't."
    }
    // trace:STORY-20 | ai:claude
    fn hint_more(&self) -> Option<&'static str> {
        Some(
            "1. `cd` into the session worktree `aida session start` printed — a sibling directory of `workspace/`.\n\
             2. Make a change there: a source file with a trace comment.\n\
             3. `git add` and `git commit` it — the commit lands on `session-work`, not the parent's branch."
        )
    }
    // trace:STORY-20 | ai:claude
    fn hint_solution(&self) -> Option<&'static str> {
        Some(
            "# cd into the session worktree (sibling dir `aida session start` printed):\n\
             mkdir -p src\n\
             cat > src/widget.rs <<'EOF'\n\
             // trace:FR-1 | ai:claude\n\
             pub fn widget() {}\n\
             EOF\n\
             git add src/widget.rs\n\
             git commit -m \"[AI:claude] feat(widget): build widget in the session worktree (FR-1)\""
        )
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        if !is_aida_initialized(workspace) {
            return VerifyResult::Pending("complete exercise 01 first".into());
        }
        if !git_branch_exists(workspace, SESSION_BRANCH) {
            return VerifyResult::Pending(
                "no session branch yet — complete exercise 27 (`aida session start`) first.".into(),
            );
        }
        let Some(base) = main_worktree_branch(workspace) else {
            return VerifyResult::Fail(
                "couldn't read the workspace's base branch from `git worktree list`.".into(),
            );
        };
        match git_commits_ahead(workspace, &base, SESSION_BRANCH) {
            Some(n) if n >= 1 => VerifyResult::Pass,
            Some(_) => VerifyResult::Pending(format!(
                "the `{SESSION_BRANCH}` branch has no commits beyond `{base}` yet — `cd` into \
                 the session worktree, make a change, and `git commit` it there."
            )),
            None => VerifyResult::Fail(
                "git couldn't count the commits on the session branch.".into(),
            ),
        }
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        // Step into the session worktree and commit work there. The
        // commit lands on `session-work`, not the parent's branch — which
        // is exactly what the verifier checks.
        let lease = demo_session_lease(workspace, SESSION_BRANCH)?;
        let wt = lease
            .worktree_path
            .ok_or_else(|| anyhow::anyhow!("demo: session lease has no worktree_path"))?;
        let wt = Path::new(&wt);
        let src = wt.join("src");
        std::fs::create_dir_all(&src)?;
        std::fs::write(
            src.join("widget.rs"),
            "// trace:FR-1 | ai:claude\npub fn widget() {}\n",
        )?;
        run(wt, "git", &["add", "src/widget.rs"])?;
        run(wt, "git", &[
            "commit", "-m",
            "[AI:claude] feat(widget): build widget in the session worktree (FR-1)",
        ])
    }
}
