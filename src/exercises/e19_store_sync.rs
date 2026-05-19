//! Exercise 19 — the orphan branch carries its own history; `aida push`
//! ships it alongside your code. trace:STORY-25 | ai:claude

use crate::exercise::{run, Exercise, VerifyResult};
use crate::verify::{git_branch_exists, git_commit_count, is_aida_initialized};
use std::path::Path;

pub struct E;

impl Exercise for E {
    fn id(&self) -> u32 { 19 }
    fn slug(&self) -> &'static str { "store-sync" }
    fn title(&self) -> &'static str { "the orphan branch has its own history" }
    fn hint(&self) -> &'static str {
        "Every `aida add` / `aida edit` auto-commits to the `aida-store` orphan branch — it's a \
         real branch with a real log. Run `aida add` to capture anything, then `git log \
         aida-store --oneline` to see the new commit land. `aida store status` shows how your \
         code commit is paired with a store SHA; `aida push --dry-run` shows the two legs \
         (code branch + orphan store) that a real `aida push` ships together."
    }
    // trace:STORY-20 | ai:claude
    fn hint_more(&self) -> Option<&'static str> {
        Some(
            "1. Run any `aida add` — every capture auto-commits to the orphan branch.\n\
             2. `git log aida-store --oneline` — watch your capture land as a commit.\n\
             3. `aida store status` shows your code paired with a store SHA."
        )
    }
    // trace:STORY-20 | ai:claude
    fn hint_solution(&self) -> Option<&'static str> {
        Some(
            "aida add --type functional --status approved --priority medium --title \"Sync the orphan store on every push\"\n\
             git log aida-store --oneline\n\
             aida store status"
        )
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        if !is_aida_initialized(workspace) {
            return VerifyResult::Pending("complete exercise 01 first".into());
        }
        if !git_branch_exists(workspace, "aida-store") {
            return VerifyResult::Fail(
                "no `aida-store` branch — complete exercise 18 first".into(),
            );
        }
        // `aida init` lays down ~4 bootstrap commits on the orphan branch;
        // every capture or edit auto-commits another. More than 4 means a
        // real store mutation has landed on the branch. trace:STORY-25
        match git_commit_count(workspace, "aida-store") {
            Some(n) if n > 4 => VerifyResult::Pass,
            Some(_) => VerifyResult::Pending(
                "the `aida-store` branch only has its bootstrap commits — run `aida add ...` \
                 to land a capture as a commit on the orphan branch.".into(),
            ),
            None => VerifyResult::Fail(
                "couldn't read the `aida-store` branch history".into(),
            ),
        }
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        // Capture a req — this auto-commits to the orphan branch — then
        // inspect the branch history and the (no-op, no-origin) push plan.
        run(workspace, "aida", &[
            "add", "--type", "functional", "--status", "approved",
            "--priority", "medium",
            "--title", "Sync the orphan store on every push",
        ])?;
        run(workspace, "git", &["log", "aida-store", "--oneline"])?;
        run(workspace, "aida", &["store", "status"])?;
        run(workspace, "aida", &["push", "--dry-run"])
    }
}
