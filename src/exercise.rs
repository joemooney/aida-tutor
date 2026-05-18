//! Exercise definition: each tutorial step is one impl of [`Exercise`].
//!
//! Verifiers inspect on-disk state — the workspace directory, the AIDA
//! store inside it, the project's git state — and report Pass / Fail /
//! Pending. They MUST NOT mutate the user's files. trace:PRIN-1 | ai:claude

use anyhow::Context;
use std::path::Path;

#[derive(Debug, Clone)]
pub enum VerifyResult {
    /// User completed the exercise; advance to the next.
    Pass,
    /// User hasn't started yet (no relevant state on disk). Surfaced as
    /// "Pending" not "Fail" so the user isn't yelled at on first run.
    Pending(String),
    /// User attempted but didn't quite hit the target. Message tells them
    /// what's wrong specifically (e.g. "no VIS-* requirement found").
    Fail(String),
}

pub trait Exercise: Sync + Send {
    /// Stable numeric id, 1-indexed.
    fn id(&self) -> u32;
    /// Short kebab-case slug (e.g. "init", "vision", "trace-comment").
    fn slug(&self) -> &'static str;
    /// One-line title shown in `aida-tutor list`.
    fn title(&self) -> &'static str;
    /// Path under `content/` for the long-form description (markdown).
    /// Default impl returns `content/NN-slug.md`.
    fn content_path(&self) -> String {
        format!("content/{:02}-{}.md", self.id(), self.slug())
    }
    /// Read the on-disk markdown content. Returns "(missing)" if absent.
    fn description(&self, repo_root: &Path) -> String {
        let p = repo_root.join(self.content_path());
        std::fs::read_to_string(&p)
            .unwrap_or_else(|_| format!("(content file missing: {})", p.display()))
    }
    /// Inspect the workspace and report state.
    fn verify(&self, workspace: &Path) -> VerifyResult;
    /// One-paragraph nudge surfaced via `aida-tutor hint`. Avoid showing
    /// the literal command — that defeats the point. trace:PRIN-3
    fn hint(&self) -> &'static str;
    /// Drive this exercise to a passing state non-interactively — run the
    /// `aida` commands (and any file writes) a learner would. Used by
    /// `aida-tutor demo` to walk every exercise without a human, so CI can
    /// gate the tutor against the live AIDA CLI surface. The default no-op
    /// is correct for read-only exercises whose `verify` already passes on
    /// prerequisite state. trace:STORY-19 | ai:claude
    fn demo(&self, _workspace: &Path) -> anyhow::Result<()> {
        Ok(())
    }
}

/// Resolve the first spec_id matching `prefix` (e.g. "FR") from the
/// workspace store. Demo impls use this instead of hardcoding an id:
/// AIDA's global ID counter numbers reqs by creation order across all
/// types, so the feature captured 4th is `FR-4`, not `FR-1`.
/// trace:STORY-19 | ai:claude
pub fn demo_spec_id(workspace: &Path, prefix: &str) -> anyhow::Result<String> {
    crate::verify::requirements_with_prefix(workspace, prefix)
        .into_iter()
        .find_map(|r| r.spec_id)
        .with_context(|| format!("demo: no {prefix}-* requirement in the store yet"))
}

/// Resolve the spec_id of the requirement whose title contains `needle`
/// (case-insensitive). The queue-cluster demos (exercises 22-24) follow
/// one task by title across capture → pickup → done, since its spec_id
/// depends on creation order. trace:STORY-27 | ai:claude
pub fn demo_req_by_title(workspace: &Path, needle: &str) -> anyhow::Result<String> {
    crate::verify::requirement_by_title(workspace, needle)
        .and_then(|r| r.spec_id)
        .with_context(|| format!("demo: no requirement whose title contains {needle:?}"))
}

/// Resolve the `aida session start` lease whose worktree is checked out
/// on `branch`. The session-cluster demos (exercises 28-30) follow one
/// session — created on a pinned branch in exercise 27 — through work →
/// inspect → end. trace:STORY-28 | ai:claude
pub fn demo_session_lease(
    workspace: &Path,
    branch: &str,
) -> anyhow::Result<crate::verify::SessionLease> {
    crate::verify::session_lease_for_branch(workspace, branch)
        .with_context(|| format!("demo: no `aida session` lease on branch {branch:?}"))
}

/// Run `program args...` with `workspace` as the working directory,
/// capturing output so demo logs stay quiet on success. Errors (with the
/// captured stdout/stderr) if the process can't spawn or exits non-zero.
/// Helper for [`Exercise::demo`] impls. trace:STORY-19 | ai:claude
pub fn run(workspace: &Path, program: &str, args: &[&str]) -> anyhow::Result<()> {
    let out = std::process::Command::new(program)
        .current_dir(workspace)
        .args(args)
        .output()
        .with_context(|| format!("spawning `{program}`"))?;
    if !out.status.success() {
        anyhow::bail!(
            "`{} {}` exited with {}\n--- stdout ---\n{}\n--- stderr ---\n{}",
            program,
            args.join(" "),
            out.status,
            String::from_utf8_lossy(&out.stdout).trim(),
            String::from_utf8_lossy(&out.stderr).trim(),
        );
    }
    Ok(())
}
