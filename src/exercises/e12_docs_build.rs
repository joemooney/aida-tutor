//! Exercise 12 — `aida docs build`. trace:STORY-12 | ai:claude

use crate::exercise::{run, Exercise, VerifyResult};
use crate::verify::{is_aida_initialized, path_exists};
use std::path::Path;

pub struct E;

impl Exercise for E {
    fn id(&self) -> u32 { 12 }
    fn slug(&self) -> &'static str { "docs-build" }
    fn title(&self) -> &'static str { "aida docs build — project canonical layer" }
    fn hint(&self) -> &'static str {
        "Run `aida docs build`. It walks the requirement graph and writes a layered markdown tree under \
         `docs/aida/`. Vision goes to `01-vision.md`, principles to `00-constitution.md`, decisions to \
         `05-decisions/`. The README at `docs/aida/README.md` is the entry point."
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        if !is_aida_initialized(workspace) {
            return VerifyResult::Pending("complete exercise 01 first".into());
        }
        if !path_exists(workspace, "docs/aida/README.md") {
            return VerifyResult::Pending(
                "no `docs/aida/README.md` yet — try `aida docs build`".into()
            );
        }
        // Confirm at least one layer file got generated alongside README.
        let layers = ["docs/aida/01-vision.md", "docs/aida/00-constitution.md"];
        if !layers.iter().any(|p| path_exists(workspace, p)) {
            return VerifyResult::Fail(
                "README.md exists but no layer file (vision/constitution). \
                 Did the build finish? Try re-running `aida docs build`.".into()
            );
        }
        VerifyResult::Pass
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        run(workspace, "aida", &["docs", "build"])
    }
}
