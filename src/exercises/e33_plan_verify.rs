//! Exercise 33 — `aida plan verify`: lint an implementation plan archived
//! under `docs/plans/` against the structured template. Cluster 6 (plans
//! + store maintenance + MCP). trace:STORY-30 | ai:claude

use crate::exercise::{run, Exercise, VerifyResult};
use crate::verify::{is_aida_initialized, plan_files};
use std::path::Path;

pub struct E;

/// Where the content + demo pin the plan file. `aida plan verify` takes
/// any path and the verifier scans the whole `docs/plans/` dir, but one
/// stable name keeps the walkthrough concrete. trace:STORY-30 | ai:claude
const PLAN: &str = "docs/plans/2026-05-18-readme-note.md";

/// The three sections `aida plan verify` treats as *required* — their
/// absence is an ERROR (not just a WARN), so a plan missing any of them
/// fails the lint. The verifier checks for the same three. Lowercase: the
/// comparison is done against a lowercased copy of the plan.
const REQUIRED_SECTIONS: [&str; 3] = ["## critical files", "## verification", "## followups"];

impl Exercise for E {
    fn id(&self) -> u32 { 33 }
    fn slug(&self) -> &'static str { "plan-verify" }
    fn title(&self) -> &'static str { "aida plan verify — lint an implementation plan" }
    fn hint(&self) -> &'static str {
        "AIDA archives implementation plans as markdown under `docs/plans/`. `aida plan verify \
         <file>` lints one against the structured template — it flags drifted `path:line` refs, \
         files the plan names that no longer exist, and missing required sections. Create \
         `docs/plans/2026-05-18-readme-note.md` with the headings `## Summary`, `## Approach`, \
         `## Critical Files`, `## Files`, `## Verification`, `## Followups`, and `## Related`, \
         point its file references at `README.md` (which exists in your workspace), then run \
         `aida plan verify docs/plans/2026-05-18-readme-note.md` until the verdict is PASS."
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        if !is_aida_initialized(workspace) {
            return VerifyResult::Pending("complete exercise 01 first".into());
        }
        let plans = plan_files(workspace);
        if plans.is_empty() {
            return VerifyResult::Pending(
                "no plan under `docs/plans/` yet — write one and run \
                 `aida plan verify docs/plans/<file>.md`.".into(),
            );
        }
        // A plan passes `aida plan verify` when it carries the three
        // required sections — WARNs about recommended sections don't fail
        // the lint. Pass if ANY plan in the dir has all three.
        for path in &plans {
            let Ok(content) = std::fs::read_to_string(path) else {
                continue;
            };
            let lc = content.to_lowercase();
            if REQUIRED_SECTIONS.iter().all(|s| lc.contains(s)) {
                return VerifyResult::Pass;
            }
        }
        VerifyResult::Fail(
            "a plan exists under `docs/plans/` but it's missing a required section — \
             `aida plan verify` errors when `## Critical Files`, `## Verification`, or \
             `## Followups` is absent. Add the missing heading(s) and re-verify.".into(),
        )
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        // Write a plan that satisfies the template, then lint it. Every
        // file reference points at `README.md` — seeded by `aida-tutor
        // reset` — so the path-existence check passes too.
        let plan = workspace.join(PLAN);
        if let Some(parent) = plan.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&plan, DEMO_PLAN)?;
        run(workspace, "aida", &["plan", "verify", PLAN])
    }
}

/// A minimal plan that passes `aida plan verify`: every required section
/// present, recommended sections partly filled, file refs that exist.
const DEMO_PLAN: &str = "\
# Note the store pairing in the README

## Summary
Add a line to `README.md` describing the AIDA store.

## Approach
Append one sentence under the project title.

## Critical Files
- `README.md` — the file the plan edits

## Files
| Action | File |
|--------|------|
| Modify | `README.md` |

## Verification
Run `aida plan verify` on this file and confirm a PASS verdict.

## Followups
- None.

## Related
- FR-1 — the feature captured in exercise 05
";
