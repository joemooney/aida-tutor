//! Exercise 07 — see the link: `aida show <id>` reveals the linked commit.
//! The emotional peak of the core loop. trace:STORY-46 | ai:claude

use crate::exercise::{demo_spec_id, run, Exercise, VerifyResult};
use crate::verify::{
    aida_show_output, commit_subject_references, is_aida_initialized, requirements_with_prefix,
    trace_comments_in_workspace,
};
use std::path::Path;

pub struct E;

impl Exercise for E {
    fn id(&self) -> u32 {
        7
    }
    fn slug(&self) -> &'static str {
        "see-link"
    }
    fn title(&self) -> &'static str {
        "aida show <id> — watch the code↔spec link appear"
    }
    fn hint(&self) -> &'static str {
        "Run `aida show FR-1` again — the same command from exercise 04. This time, because \
         you've written a `trace:FR-1` comment and committed it with `(FR-1)`, the output grows \
         a `Git linkage` section listing your commit and the file it traced. The spec and the \
         code found each other. That two-way link is the whole point of AIDA."
    }
    fn hint_more(&self) -> Option<&'static str> {
        Some(
            "1. Make sure exercises 05 (trace comment) and 06 (the `(FR-1)` commit) are done.\n\
             2. Re-run `aida show FR-1`.\n\
             3. Look for the `Git linkage` block — `Commits` and `Files traced` — near the bottom.",
        )
    }
    fn hint_solution(&self) -> Option<&'static str> {
        Some("aida show FR-1")
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        if !is_aida_initialized(workspace) {
            return VerifyResult::Pending("complete exercise 01 first".into());
        }
        // The linkage only renders once three things exist: the feature
        // (ex 02), a trace comment pointing at it (ex 05), and a commit
        // whose subject references it (ex 06). Nudge precisely for whichever
        // is missing.
        let Some(spec) = requirements_with_prefix(workspace, "FR")
            .into_iter()
            .find_map(|r| r.spec_id)
        else {
            return VerifyResult::Pending(
                "no FR-* requirement yet — capture the feature in exercise 02".into(),
            );
        };
        if trace_comments_in_workspace(workspace)
            .iter()
            .all(|s| !s.starts_with("FR"))
        {
            return VerifyResult::Pending(
                "no `trace:FR-...` comment in workspace/ yet — write one (exercise 05)".into(),
            );
        }
        if !commit_subject_references(workspace, &spec) {
            return VerifyResult::Pending(format!(
                "no commit references `({spec})` yet — commit your traced file with `({spec})` \
                 at the end of the message (exercise 06)"
            ));
        }
        // The aha: the running CLI now renders the Git-linkage section.
        let Some(out) = aida_show_output(workspace, &spec) else {
            return VerifyResult::Fail(format!("`aida show {spec}` did not run cleanly"));
        };
        let has_linkage = out.contains("Git linkage")
            && (out.contains("Files traced") || out.contains("Commits"));
        if !has_linkage {
            return VerifyResult::Fail(format!(
                "`aida show {spec}` ran but shows no `Git linkage` section — make sure your \
                 commit subject ends with `({spec})` and the committed file has a `trace:{spec}` comment"
            ));
        }
        VerifyResult::Pass
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        let fr = demo_spec_id(workspace, "FR")?;
        run(workspace, "aida", &["show", &fr])
    }
}
