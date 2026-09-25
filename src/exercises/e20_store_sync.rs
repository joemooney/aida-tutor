//! Exercise 20 — the orphan branch carries its own history; `aida push`
//! ships it alongside your code. trace:STORY-25 | ai:claude

use crate::exercise::{run, Exercise, VerifyResult};
use crate::progress::Progress;
use crate::verify::{git_branch_exists, git_commit_count, is_aida_initialized};
use std::path::Path;

use super::e19_distributed_store::DEMO_STORE_BASELINE;

pub struct E;

impl Exercise for E {
    fn id(&self) -> u32 {
        20
    }
    fn slug(&self) -> &'static str {
        "store-sync"
    }
    fn title(&self) -> &'static str {
        "the orphan branch has its own history"
    }
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
             3. `aida store status` shows your code paired with a store SHA.",
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
                "no `aida-store` branch — complete exercise 19 first".into(),
            );
        }
        // Compare against the count captured when exercise 19 passed. A
        // fixed bootstrap count is coupled to aida init internals and could
        // false-pass after a future init changes. trace:TASK-3
        let baseline = workspace
            .parent()
            .and_then(|repo_root| Progress::load(repo_root).ok())
            .and_then(|progress| progress.store_commit_baseline)
            .or_else(|| {
                std::fs::read_to_string(workspace.join(DEMO_STORE_BASELINE))
                    .ok()
                    .and_then(|value| value.trim().parse().ok())
            });
        let Some(baseline) = baseline else {
            return VerifyResult::Pending(
                "no exercise-19 store baseline recorded — verify exercise 19 again, then run this exercise."
                    .into(),
            );
        };
        match git_commit_count(workspace, "aida-store") {
            Some(n) if n > baseline => VerifyResult::Pass,
            Some(_) => VerifyResult::Pending(
                "the `aida-store` branch has not changed since exercise 19 — run `aida add ...` \
                 to land a capture as a commit on the orphan branch."
                    .into(),
            ),
            None => VerifyResult::Fail("couldn't read the `aida-store` branch history".into()),
        }
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        // Capture a req — this auto-commits to the orphan branch — then
        // inspect the branch history and the (no-op, no-origin) push plan.
        run(
            workspace,
            "aida",
            &[
                "add",
                "--type",
                "functional",
                "--status",
                "approved",
                "--priority",
                "medium",
                "--title",
                "Sync the orphan store on every push",
            ],
        )?;
        run(workspace, "git", &["log", "aida-store", "--oneline"])?;
        run(workspace, "aida", &["store", "status"])?;
        run(workspace, "aida", &["push", "--dry-run"])
    }
}
