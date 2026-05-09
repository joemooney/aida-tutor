//! Exercise 15 — `aida show <id> --comments`. trace:STORY-15 | ai:claude

use crate::exercise::{Exercise, VerifyResult};
use crate::verify::is_aida_initialized;
use std::path::Path;

pub struct E;

impl Exercise for E {
    fn id(&self) -> u32 { 15 }
    fn slug(&self) -> &'static str { "show-comments" }
    fn title(&self) -> &'static str { "aida show <id> --comments — read the audit trail" }
    fn hint(&self) -> &'static str {
        "Run `aida show <FR-id> --comments`. You'll see the full record + every comment with author + \
         timestamp + content. This is what survives in the audit log when teammates ask 'when did we \
         decide that?' six months later."
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        if !is_aida_initialized(workspace) {
            return VerifyResult::Pending("complete exercise 01 first".into());
        }
        VerifyResult::Pass
    }
}
