//! Exercise 39 — safe dispatch previews. trace:STORY-58 | ai:codex

use crate::exercise::{demo_spec_id, run, verify_invocation, Exercise, VerifyResult};
use crate::verify::is_aida_initialized;
use std::path::Path;

pub struct E;

impl Exercise for E {
    fn id(&self) -> u32 {
        39
    }
    fn slug(&self) -> &'static str {
        "dispatch-dryrun"
    }
    fn title(&self) -> &'static str {
        "do / zen / ship — preview the human contract"
    }
    fn hint(&self) -> &'static str {
        "AIDA now has one-verb workflow entry points. Preview them before allowing side effects: `aida do <SPEC>`, `aida zen <SPEC> --dry-run`, and `aida ship <SPEC> --dry-run`. The preview should make the execution mode and human checkpoint explicit."
    }
    fn hint_more(&self) -> Option<&'static str> {
        Some("1. Identify the feature spec.\n2. Run `aida zen <SPEC> --dry-run` and read the suitability gate.\n3. Run `aida ship <SPEC> --dry-run`; do not run a real ship from the tutorial workspace.")
    }
    fn hint_solution(&self) -> Option<&'static str> {
        Some("aida do FR-1 --mode operator\naida zen FR-1 --dry-run\naida ship FR-1 --dry-run")
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        if !is_aida_initialized(workspace) {
            return VerifyResult::Pending("complete exercise 01 first".into());
        }
        let zen = verify_invocation(
            workspace,
            "zen",
            &["--dry-run"],
            "run `aida zen <SPEC> --dry-run`",
        );
        if !matches!(zen, VerifyResult::Pass) {
            return zen;
        }
        verify_invocation(
            workspace,
            "ship",
            &["--dry-run"],
            "run `aida ship <SPEC> --dry-run`",
        )
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        run(workspace, "aida", &["focus", "--clear"])?;
        let fr = demo_spec_id(workspace, "FR")?;
        std::env::set_var("AIDA_SESSION_ROLE", "advisor");
        // The legacy feature fixture is intentionally terse for the novice
        // exercises. Give the current suitability gate enough behavior to
        // describe before previewing zen. trace:STORY-58 | ai:codex
        run(
            workspace,
            "aida",
            &[
                "edit",
                &fr,
                "--description",
                "Demonstrate a small feature with a traceable implementation and reviewable finish.",
            ],
        )?;
        // `aida ship` intentionally refuses the default branch. Give the
        // safe preview a feature branch without opening a PR or changing the
        // remote. trace:STORY-58 | ai:codex
        run(
            workspace,
            "git",
            &["checkout", "-B", "tutor-dispatch-preview"],
        )?;
        run(workspace, "aida", &["do", &fr, "--mode", "operator"])?;
        run(workspace, "aida", &["zen", &fr, "--dry-run"])?;
        run(workspace, "aida", &["ship", &fr, "--dry-run"])
    }
}
