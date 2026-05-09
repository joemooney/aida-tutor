//! Progress tracking. Persisted to `.aida-tutor-progress.toml` in the
//! tutor repo root (NOT the workspace) so a `aida-tutor reset` doesn't
//! also wipe the user's progress. trace:PRIN-2 | ai:claude

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const FILE: &str = ".aida-tutor-progress.toml";

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Progress {
    /// Highest exercise id the learner has completed (0 = none yet).
    #[serde(default)]
    pub completed_through: u32,
    /// Total exercises completed (== completed_through after a clean walk;
    /// can diverge if exercises are added later — the user keeps credit).
    #[serde(default)]
    pub completed: Vec<u32>,
}

impl Progress {
    pub fn path(repo_root: &Path) -> PathBuf {
        repo_root.join(FILE)
    }

    pub fn load(repo_root: &Path) -> Result<Self> {
        let p = Self::path(repo_root);
        if !p.exists() {
            return Ok(Self::default());
        }
        let s = std::fs::read_to_string(&p)
            .with_context(|| format!("reading {}", p.display()))?;
        toml::from_str(&s).with_context(|| format!("parsing {}", p.display()))
    }

    pub fn save(&self, repo_root: &Path) -> Result<()> {
        let p = Self::path(repo_root);
        let s = toml::to_string_pretty(self)?;
        std::fs::write(&p, s).with_context(|| format!("writing {}", p.display()))?;
        Ok(())
    }

    pub fn is_completed(&self, id: u32) -> bool {
        self.completed.contains(&id)
    }

    pub fn record_completion(&mut self, id: u32) {
        if !self.completed.contains(&id) {
            self.completed.push(id);
            self.completed.sort_unstable();
        }
        if id > self.completed_through {
            self.completed_through = id;
        }
    }

    /// Lowest id that's NOT yet completed. None if everything's done.
    pub fn current(&self, total_exercises: u32) -> Option<u32> {
        for id in 1..=total_exercises {
            if !self.is_completed(id) {
                return Some(id);
            }
        }
        None
    }
}
