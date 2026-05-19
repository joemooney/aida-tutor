//! Exercise 34 — `aida db info` / `aida db check`: audit the requirement
//! store's health. Cluster 6 (plans + store maintenance + MCP).
//! trace:STORY-30 | ai:claude

use crate::exercise::{run, Exercise, VerifyResult};
use crate::verify::{duplicate_spec_ids, is_aida_initialized};
use std::path::Path;

pub struct E;

impl Exercise for E {
    fn id(&self) -> u32 { 34 }
    fn slug(&self) -> &'static str { "store-audit" }
    fn title(&self) -> &'static str { "aida db info / db check — audit the store" }
    fn hint(&self) -> &'static str {
        "The requirement store is plain YAML on the orphan branch — and it's auditable. \
         `aida db info` is the stat view: storage backend, store path, requirement count, the \
         agreed-id blocks, and whether the store worktree has uncommitted changes. \
         `aida db check --collisions` is the fsck: it scans for two requirements claiming the \
         same display id. Run both from `workspace/` and read the output — a healthy store \
         reports no collisions."
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        if !is_aida_initialized(workspace) {
            return VerifyResult::Pending("complete exercise 01 first".into());
        }
        // The same scan `aida db check --collisions` runs: no two
        // requirements may claim the same spec id. A tutorial store is
        // always healthy here — this read-only exercise verifies on the
        // store being collision-free, the honest proxy for "you ran the
        // audit and it came back clean".
        let dups = duplicate_spec_ids(workspace);
        if !dups.is_empty() {
            return VerifyResult::Fail(format!(
                "the store has colliding spec id(s): {} — `aida db check --collisions` flags \
                 this. The tutorial store should be clean; something corrupted it.",
                dups.join(", ")
            ));
        }
        VerifyResult::Pass
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        // Read-only audit — the two commands a learner runs to inspect
        // store health.
        run(workspace, "aida", &["db", "info"])?;
        run(workspace, "aida", &["db", "check", "--collisions"])
    }
}
