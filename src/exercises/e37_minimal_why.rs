//! Exercise 37 — markdown-first init and `aida why`. trace:STORY-56 | ai:codex

use crate::exercise::{run, Exercise, VerifyResult};
use std::path::Path;

pub struct E;

impl Exercise for E {
    fn id(&self) -> u32 {
        37
    }
    fn slug(&self) -> &'static str {
        "minimal-why"
    }
    fn title(&self) -> &'static str {
        "markdown-first init — ask code why it exists"
    }
    fn hint(&self) -> &'static str {
        "Try AIDA's zero-machine first run in a throwaway directory: `aida init --minimal`. It creates a `specs/` folder and a tiny traced example. Then run `aida why example.py:2` to resolve the trace without a git-canonical store."
    }
    fn hint_more(&self) -> Option<&'static str> {
        Some("1. Make a temporary directory under workspace/.\n2. Run `aida init --minimal`.\n3. Run `aida why example.py:2`; read the plain-markdown fallback message.")
    }
    fn hint_solution(&self) -> Option<&'static str> {
        Some("mkdir minimal-demo && cd minimal-demo\naida init --minimal\naida why example.py:2")
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        let root = workspace.join("minimal-demo");
        if !root.join("specs").join("EXAMPLE-1.md").exists() || !root.join("example.py").exists() {
            return VerifyResult::Pending(
                "create minimal-demo with `aida init --minimal` first".into(),
            );
        }
        let output = std::process::Command::new("aida")
            .current_dir(&root)
            .args(["why", "example.py:2"])
            .output();
        match output {
            Ok(out)
                if out.status.success()
                    && String::from_utf8_lossy(&out.stdout).contains("EXAMPLE-1") =>
            {
                VerifyResult::Pass
            }
            Ok(_) => VerifyResult::Fail(
                "`aida why example.py:2` did not resolve the example trace".into(),
            ),
            Err(err) => VerifyResult::Fail(format!("could not run `aida why`: {err}")),
        }
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        let root = workspace.join("minimal-demo");
        std::fs::create_dir_all(&root)?;
        run(
            &root,
            "aida",
            &["init", "--minimal", "--no-post-hooks", "--no-agent-config"],
        )?;
        run(&root, "aida", &["why", "example.py:2"])
    }
}
