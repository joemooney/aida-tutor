//! Exercise 11 — commit with the AIDA format. trace:STORY-11 | ai:claude

use crate::exercise::{Exercise, VerifyResult};
use crate::verify::{is_aida_initialized, last_commit_message};
use std::path::Path;

pub struct E;

impl Exercise for E {
    fn id(&self) -> u32 { 11 }
    fn slug(&self) -> &'static str { "aida-commit" }
    fn title(&self) -> &'static str { "commit with `[AI:tool] type(scope): description (REQ-ID)`" }
    fn hint(&self) -> &'static str {
        "Stage the file you wrote in exercise 10 and commit it. The commit message must follow \
         `[AI:tool] type(scope): description (REQ-ID)` — e.g. `[AI:claude] feat(stub): scaffold (FR-4)`. \
         AIDA's commit-msg hook will validate the format and ✓ when it matches."
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        if !is_aida_initialized(workspace) {
            return VerifyResult::Pending("complete exercise 01 first".into());
        }
        let Some(msg) = last_commit_message(workspace) else {
            return VerifyResult::Pending("no commits in workspace/ yet".into());
        };
        let first_line = msg.lines().next().unwrap_or("");

        // Quick checks against the same shape AIDA's commit-msg hook validates.
        let conv_re = regex::Regex::new(
            r"^(\[AI:[a-zA-Z]+(:(high|med|low))?\]\s+)?(feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert)(\([a-zA-Z0-9_-]+\))?:\s+.+",
        ).unwrap();
        let req_re = regex::Regex::new(
            r"\([A-Z]+(-[0-9]+){1,2}(,[[:space:]]*[A-Z]+(-[0-9]+){1,2})*\)$"
        ).unwrap();

        if !conv_re.is_match(first_line) {
            return VerifyResult::Fail(
                format!(
                    "last commit `{}` doesn't match `[AI:tool]? type(scope)?: description ...`",
                    first_line
                )
            );
        }
        if !req_re.is_match(first_line) {
            return VerifyResult::Fail(
                format!(
                    "last commit `{}` is missing a `(REQ-ID)` at the end",
                    first_line
                )
            );
        }
        VerifyResult::Pass
    }
}
