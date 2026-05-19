//! Exercise 27 — start a scoped session: `aida session start` forks a
//! sibling git worktree on its own branch and records a lease.
//! Cluster 4 (sessions + worktrees). trace:STORY-28 | ai:claude

use crate::exercise::{demo_spec_id, run, Exercise, VerifyResult};
use crate::verify::{git_worktrees, is_aida_initialized, session_lease_for_branch};
use std::path::Path;

pub struct E;

/// The branch name the session cluster pins. Exercises 27-30 follow one
/// session across start → work → inspect → end; the content tells the
/// learner to use this branch verbatim, and the verifiers key on it.
/// trace:STORY-28 | ai:claude
pub const SESSION_BRANCH: &str = "session-work";

impl Exercise for E {
    fn id(&self) -> u32 { 27 }
    fn slug(&self) -> &'static str { "session-start" }
    fn title(&self) -> &'static str { "aida session start — a scoped worktree" }
    fn hint(&self) -> &'static str {
        "A *session* claims a slice of work and gives it its own room. `aida session start \
         --owns <SCOPE> --branch <NAME>` forks a git worktree — a second checkout of the repo, \
         a sibling directory of `workspace/` — onto a fresh branch, symlinks the AIDA store \
         into it so requirements stay shared, and writes a lease at `.aida/sessions/<id>.toml` \
         recording who owns what. Scope it to the feature from exercise 05 (`FR-1` — confirm \
         with `aida list --type functional`) and pin the branch: `aida session start --owns \
         FR-1 --branch session-work`."
    }
    // trace:STORY-20 | ai:claude
    fn hint_more(&self) -> Option<&'static str> {
        Some(
            "1. The feature from exercise 05 is FR-1 — confirm with `aida list --type functional`.\n\
             2. Run `aida session start` with `--owns FR-1` and `--branch session-work`.\n\
             3. It forks a sibling worktree and writes a lease under `.aida/sessions/`."
        )
    }
    // trace:STORY-20 | ai:claude
    fn hint_solution(&self) -> Option<&'static str> {
        Some("aida session start --owns FR-1 --branch session-work")
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        if !is_aida_initialized(workspace) {
            return VerifyResult::Pending("complete exercise 01 first".into());
        }
        let has_lease = session_lease_for_branch(workspace, SESSION_BRANCH).is_some();
        // A worktree counts only if git still has it registered AND its
        // directory is actually on disk — a registered-but-deleted dir is
        // the half-removed state a bare `rm -rf` leaves behind.
        let has_worktree = git_worktrees(workspace)
            .into_iter()
            .filter(|w| w.branch.as_deref() == Some(SESSION_BRANCH))
            .any(|w| Path::new(&w.path).is_dir());
        match (has_lease, has_worktree) {
            (false, false) => VerifyResult::Pending(format!(
                "no scoped session yet — run `aida session start --owns FR-1 --branch {SESSION_BRANCH}`."
            )),
            (true, false) => VerifyResult::Fail(format!(
                "a lease for `{SESSION_BRANCH}` exists but git has no worktree on that branch — \
                 the worktree was removed without `aida session end`. End the session to clear \
                 the stale lease, then start fresh."
            )),
            (false, true) => VerifyResult::Fail(format!(
                "git has a `{SESSION_BRANCH}` worktree but there's no AIDA session lease — it \
                 looks made with bare `git worktree add`. Use `aida session start` so the \
                 session is tracked and `aida session leases` can see it."
            )),
            (true, true) => VerifyResult::Pass,
        }
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        // Scope the session to the feature captured in exercise 05. Its
        // spec_id depends on creation order, so resolve it rather than
        // hardcoding `FR-1`. The branch is pinned so exercises 28-30 can
        // follow this one session.
        let feature = demo_spec_id(workspace, "FR")?;
        run(workspace, "aida", &[
            "session", "start", "--owns", &feature, "--branch", SESSION_BRANCH,
        ])
    }
}
