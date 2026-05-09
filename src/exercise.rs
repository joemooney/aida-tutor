//! Exercise definition: each tutorial step is one impl of [`Exercise`].
//!
//! Verifiers inspect on-disk state — the workspace directory, the AIDA
//! store inside it, the project's git state — and report Pass / Fail /
//! Pending. They MUST NOT mutate the user's files. trace:PRIN-1 | ai:claude

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
}
