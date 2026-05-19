//! Exercise 18 — the distributed store: orphan branch + linked worktree.
//! trace:STORY-25 | ai:claude

use crate::exercise::{run, Exercise, VerifyResult};
use crate::verify::{
    cache_db_is_valid_sqlite, git_branch_exists, is_aida_initialized, is_linked_worktree,
};
use std::path::Path;

pub struct E;

impl Exercise for E {
    fn id(&self) -> u32 { 18 }
    fn slug(&self) -> &'static str { "distributed-store" }
    fn title(&self) -> &'static str { "the distributed store — orphan branch + worktree" }
    fn hint(&self) -> &'static str {
        "`aida init` (exercise 01) built a distributed, git-canonical store. It lives on a \
         separate orphan branch called `aida-store`, checked out as a linked worktree at \
         `.aida-store/`. Run `git branch`, `git worktree list`, and `aida cache status` to see \
         all three pieces: the branch, the worktree, and the rebuildable `.aida/cache.db`."
    }
    // trace:STORY-20 | ai:claude
    fn hint_more(&self) -> Option<&'static str> {
        Some(
            "1. `git branch` — spot the `aida-store` orphan branch `aida init` created.\n\
             2. `git worktree list` — see `.aida-store/` checked out on that branch.\n\
             3. `aida cache status` — materializes `.aida/cache.db` if it isn't there yet."
        )
    }
    // trace:STORY-20 | ai:claude
    fn hint_solution(&self) -> Option<&'static str> {
        Some(
            "git branch --list aida-store\n\
             git worktree list\n\
             aida cache status"
        )
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        if !is_aida_initialized(workspace) {
            return VerifyResult::Pending("complete exercise 01 first".into());
        }
        if !git_branch_exists(workspace, "aida-store") {
            return VerifyResult::Fail(
                "no `aida-store` branch in workspace/ — the orphan store branch is missing. \
                 `aida init` should have created it.".into(),
            );
        }
        if !is_linked_worktree(workspace, ".aida-store") {
            return VerifyResult::Fail(
                "`.aida-store/` is not a linked git worktree (no `.git` pointer file). The \
                 distributed store is a worktree checked out on the `aida-store` branch.".into(),
            );
        }
        if !cache_db_is_valid_sqlite(workspace) {
            return VerifyResult::Pending(
                "no `.aida/cache.db` yet — run `aida cache status` to materialize the cache, \
                 then verify again.".into(),
            );
        }
        VerifyResult::Pass
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        // Read-only inspection — the commands a learner runs to see the
        // three pieces of the distributed store.
        run(workspace, "git", &["branch", "--list", "aida-store"])?;
        run(workspace, "git", &["worktree", "list"])?;
        run(workspace, "aida", &["db", "path"])?;
        run(workspace, "aida", &["cache", "status"])
    }
}
