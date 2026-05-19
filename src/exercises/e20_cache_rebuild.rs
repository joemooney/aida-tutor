//! Exercise 20 — the cache is disposable; rebuild it from the git store.
//! trace:STORY-25 | ai:claude

use crate::exercise::{run, Exercise, VerifyResult};
use crate::verify::{cache_db_is_valid_sqlite, is_aida_initialized};
use std::path::Path;

pub struct E;

impl Exercise for E {
    fn id(&self) -> u32 { 20 }
    fn slug(&self) -> &'static str { "cache-rebuild" }
    fn title(&self) -> &'static str { "aida cache rebuild — the cache is disposable" }
    fn hint(&self) -> &'static str {
        "`.aida/cache.db` is a SQLite projection of the git store — gitignored and fully \
         rebuildable. Delete it (simulating a fresh clone, or a cache that drifted), then run \
         `aida cache rebuild` and confirm `aida list` still works. The lesson: the orphan \
         branch is canonical; the cache is a throwaway index you can regenerate any time."
    }
    // trace:STORY-20 | ai:claude
    fn hint_more(&self) -> Option<&'static str> {
        Some(
            "1. Delete `.aida/cache.db` — simulating a fresh clone that has no cache.\n\
             2. Run `aida cache rebuild` to regenerate it from the orphan branch.\n\
             3. `aida list` proves reads work again."
        )
    }
    // trace:STORY-20 | ai:claude
    fn hint_solution(&self) -> Option<&'static str> {
        Some(
            "# from inside workspace/:\n\
             rm -f .aida/cache.db\n\
             aida cache rebuild\n\
             aida list"
        )
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        if !is_aida_initialized(workspace) {
            return VerifyResult::Pending("complete exercise 01 first".into());
        }
        if !cache_db_is_valid_sqlite(workspace) {
            return VerifyResult::Fail(
                "`.aida/cache.db` is missing or not a valid SQLite database. Run \
                 `aida cache rebuild` to regenerate it from the git store.".into(),
            );
        }
        VerifyResult::Pass
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        // Simulate a missing cache (a fresh clone has no `.aida/cache.db`),
        // then rebuild it from the canonical git store and confirm reads
        // work again.
        let cache = workspace.join(".aida").join("cache.db");
        if cache.exists() {
            std::fs::remove_file(&cache)?;
        }
        run(workspace, "aida", &["cache", "rebuild"])?;
        run(workspace, "aida", &["list"])
    }
}
