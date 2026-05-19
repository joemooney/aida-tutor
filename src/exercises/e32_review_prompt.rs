//! Exercise 32 — `aida review prompt`: collect requirements' acceptance
//! criteria into a markdown review brief. Cluster 5 (code review +
//! commit pairing). trace:STORY-29 | ai:claude

use crate::exercise::{demo_spec_id, run, Exercise, VerifyResult};
use crate::verify::is_aida_initialized;
use std::path::Path;

pub struct E;

/// Where the content tells the learner to `--write` the generated brief.
/// `aida review prompt` defaults to stdout, which leaves nothing on disk
/// to inspect — so the exercise pins the output to a file the verifier
/// can read back. trace:STORY-29 | ai:claude
const BRIEF: &str = "review-brief.md";

impl Exercise for E {
    fn id(&self) -> u32 { 32 }
    fn slug(&self) -> &'static str { "review-prompt" }
    fn title(&self) -> &'static str { "aida review prompt — specs into a review brief" }
    fn hint(&self) -> &'static str {
        "`aida review prompt --specs <ids>` gathers the acceptance criteria of one or more \
         requirements into a markdown brief — exactly what to hand a reviewer (human or agent) \
         so they check against the spec, not a paraphrase. Point it at the feature from \
         exercise 05 and capture the output to a file with `--write`: `aida review prompt \
         --specs FR-1 --write review-brief.md`. Then open `review-brief.md` to read the brief."
    }
    // trace:STORY-20 | ai:claude
    fn hint_more(&self) -> Option<&'static str> {
        Some(
            "1. The feature from exercise 05 is FR-1.\n\
             2. Run `aida review prompt --specs FR-1`.\n\
             3. Add `--write review-brief.md` so the brief lands on disk for the verifier to read."
        )
    }
    // trace:STORY-20 | ai:claude
    fn hint_solution(&self) -> Option<&'static str> {
        Some("aida review prompt --specs FR-1 --write review-brief.md")
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        if !is_aida_initialized(workspace) {
            return VerifyResult::Pending("complete exercise 01 first".into());
        }
        let Ok(content) = std::fs::read_to_string(workspace.join(BRIEF)) else {
            return VerifyResult::Pending(format!(
                "no `{BRIEF}` yet — from `workspace/`, run \
                 `aida review prompt --specs FR-1 --write {BRIEF}`."
            ));
        };
        // A real generated brief has the `# Review Prompt` header, a
        // `## What to verify` checklist, and a `### <SPEC-ID>` section per
        // spec. Check all three so a stray / empty file can't pass.
        let has_header = content.contains("# Review Prompt");
        let has_checklist = content.contains("## What to verify");
        let spec_section = regex::Regex::new(r"(?m)^### [A-Z]+-[0-9]+").unwrap();
        if !has_header || !has_checklist || !spec_section.is_match(&content) {
            return VerifyResult::Fail(format!(
                "`{BRIEF}` doesn't look like a generated review brief — it should have a \
                 `# Review Prompt` header and a `## What to verify` section with a \
                 `### <SPEC-ID>` entry. Regenerate it: \
                 `aida review prompt --specs FR-1 --write {BRIEF}`."
            ));
        }
        VerifyResult::Pass
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        // Brief the feature captured in exercise 05; its spec_id depends
        // on creation order, so resolve it rather than hardcoding `FR-1`.
        let fr = demo_spec_id(workspace, "FR")?;
        run(workspace, "aida", &[
            "review", "prompt", "--specs", &fr, "--write", BRIEF,
        ])
    }
}
