//! Exercise 10 — write code with a trace comment. trace:STORY-10 | ai:claude

use crate::exercise::{Exercise, VerifyResult};
use crate::verify::{is_aida_initialized, trace_comments_in_workspace};
use std::path::Path;

pub struct E;

impl Exercise for E {
    fn id(&self) -> u32 { 10 }
    fn slug(&self) -> &'static str { "trace-comment" }
    fn title(&self) -> &'static str { "write code with a trace:FR-1 comment" }
    fn hint(&self) -> &'static str {
        "Create any source file under workspace/ (e.g. `src/something.rs`) containing a comment of the \
         form `trace:<your-FR-id> | ai:claude`. The format is `trace:<SPEC-ID> | ai:<tool>[:<conf>]` — \
         confidence defaults to high when omitted. This is the link from code to spec that the rest of \
         the AIDA tooling reads."
    }
    // trace:STORY-20 | ai:claude
    fn hint_more(&self) -> Option<&'static str> {
        Some(
            "1. Create any source file under `workspace/` — `src/parser.rs`, say.\n\
             2. Add a comment line of the form `// trace:FR-1 | ai:claude` (use your FR id).\n\
             3. Save it — the verifier scans every file under `workspace/`."
        )
    }
    // trace:STORY-20 | ai:claude
    fn hint_solution(&self) -> Option<&'static str> {
        Some(
            "# from inside workspace/:\n\
             mkdir -p src\n\
             cat > src/parser.rs <<'EOF'\n\
             // trace:FR-1 | ai:claude\n\
             pub fn parse_json() {}\n\
             EOF"
        )
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        if !is_aida_initialized(workspace) {
            return VerifyResult::Pending("complete exercise 01 first".into());
        }
        let traces = trace_comments_in_workspace(workspace);
        let fr_traces: Vec<_> = traces.iter().filter(|s| s.starts_with("FR")).collect();
        if fr_traces.is_empty() {
            return VerifyResult::Pending(
                "no `trace:FR-...` comment found in any file under workspace/ — \
                 add one in any source file (Rust, Python, JS, anything)".into()
            );
        }
        VerifyResult::Pass
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        let src = workspace.join("src");
        std::fs::create_dir_all(&src)?;
        std::fs::write(
            src.join("parser.rs"),
            "// trace:FR-1 | ai:claude\npub fn parse_json() {}\n",
        )?;
        Ok(())
    }
}
