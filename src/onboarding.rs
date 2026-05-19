//! The first-contact onboarding slice (EPIC-5).
//!
//! A thin, guided track — separate from the 35-exercise registry — that
//! delivers one round-trip "the project remembers" wow in 10-15 minutes.
//! Built only on AIDA's stable surface (spec graph, capture, trace
//! comments, `aida show`, commits); the agent-collaboration layer is
//! deliberately out of scope.
//!
//! Unlike the `Exercise` track this keeps NO progress file: the learner's
//! position is recomputed from on-disk workspace state on every run, so
//! the project itself is the only memory — which is exactly the point the
//! slice is making. trace:EPIC-5 | ai:claude

use anyhow::{bail, Context, Result};
use colored::Colorize;
use std::path::Path;

use crate::verify;

/// Marker file proving `workspace/` holds the onboarding scratch project
/// rather than the exercise-track playground. trace:STORY-31 | ai:claude
const SCRATCH_MARKER: &str = "greet.py";

/// Content files for each screen, in render order. The screen index is the
/// number of checkpoints passed (0..=5): the why-screen rides with init on
/// screen 0, and the cold-agent + signpost ride together as the finale.
/// trace:STORY-32 | ai:claude trace:STORY-34 | ai:claude
const SCREENS: &[&[&str]] = &[
    &["00-why", "01-init"],
    &["02-capture"],
    &["03-implement"],
    &["04-commit"],
    &["05-reveal"],
    &["06-cold-agent", "07-signpost"],
];

/// A state-changing checkpoint, verified from on-disk state alone. The
/// slice's read-only payoff steps (`aida show`, the cold-agent query) are
/// deliberately NOT checkpoints — they are the experiential wow, observed
/// in the TASK-2 moderated human test, not Rust-verified (AC-5).
/// trace:STORY-33 | ai:claude
struct Checkpoint {
    /// Short label shown in the progress panel.
    label: &'static str,
    /// Predicate over the seeded workspace — true once the learner has
    /// done the step.
    met: fn(&Path) -> bool,
    /// One-line nudge shown in the footer while the checkpoint is pending.
    nudge: &'static str,
}

/// The five checkpoints, in order. `passed` (below) counts how many verify
/// true from the front — that count is the learner's position in the
/// slice. trace:STORY-33 | ai:claude
fn checkpoints() -> [Checkpoint; 5] {
    [
        Checkpoint {
            label: "project initialized (aida init)",
            met: verify::is_aida_initialized,
            nudge: "Run `aida init` inside workspace/ to give the project a store.",
        },
        Checkpoint {
            label: "intent captured as a spec",
            met: cp_spec_captured,
            nudge: "Capture the feature: `aida add --type functional --status approved --title \"...\"`.",
        },
        Checkpoint {
            label: "trace comment links code to spec",
            met: cp_trace_present,
            nudge: "Add a `# trace:FR-1 | ai:claude` comment next to the new code.",
        },
        Checkpoint {
            label: "commit references the spec",
            met: cp_commit_links,
            nudge: "Commit with the spec id in parens, e.g. `... (FR-1)`.",
        },
        Checkpoint {
            label: "feature closed out (status flipped)",
            met: cp_status_flipped,
            nudge: "Close the loop: `aida edit FR-1 --status completed`.",
        },
    ]
}

/// Spec id of the first functional requirement in the workspace store, if
/// any. The trace/commit/status checkpoints resolve the id this way rather
/// than hardcoding `FR-1`, so the slice still verifies if AIDA's counter
/// ever lands a different number. trace:STORY-33 | ai:claude
fn fr_spec_id(workspace: &Path) -> Option<String> {
    verify::requirements_with_prefix(workspace, "FR")
        .into_iter()
        .find_map(|r| r.spec_id)
}

/// Checkpoint 2 — the captured intent landed as an `FR-*` object in the
/// store. trace:STORY-33 | ai:claude
fn cp_spec_captured(workspace: &Path) -> bool {
    fr_spec_id(workspace).is_some()
}

/// Checkpoint 3 — the scratch code carries a `trace:<FR>` comment pointing
/// at the captured spec. trace:STORY-33 | ai:claude
fn cp_trace_present(workspace: &Path) -> bool {
    match fr_spec_id(workspace) {
        Some(id) => verify::trace_comments_in_workspace(workspace)
            .iter()
            .any(|t| *t == id),
        None => false,
    }
}

/// Checkpoint 4 — a commit subject references the captured spec.
/// trace:STORY-33 | ai:claude
fn cp_commit_links(workspace: &Path) -> bool {
    match fr_spec_id(workspace) {
        Some(id) => verify::commit_subject_references(workspace, &id),
        None => false,
    }
}

/// Checkpoint 5 — the captured spec has left `approved`: an `FR-*` is now
/// `completed` (or `done`). trace:STORY-33 | ai:claude
fn cp_status_flipped(workspace: &Path) -> bool {
    verify::requirements_with_prefix(workspace, "FR")
        .iter()
        .any(|r| {
            r.status
                .as_deref()
                .map(|s| s.eq_ignore_ascii_case("completed") || s.eq_ignore_ascii_case("done"))
                .unwrap_or(false)
        })
}

/// Run the onboarding slice: seed the workspace if needed, figure out how
/// far the learner has got from on-disk state, and render the current
/// screen. `reset` wipes `workspace/` and restarts from screen 0.
/// trace:STORY-31 | ai:claude
pub fn run(workspace: &Path, repo_root: &Path, reset: bool) -> Result<()> {
    if reset {
        reseed(workspace, repo_root)?;
        println!(
            "{} workspace reset — the `greet` scratch project is fresh.",
            "✓".green()
        );
        println!();
    } else {
        ensure_seeded(workspace, repo_root)?;
    }

    let cps = checkpoints();
    // `passed` stops at the first unmet checkpoint, so the slice is walked
    // in order even if the learner does steps out of sequence.
    let passed = cps.iter().take_while(|c| (c.met)(workspace)).count();

    print_header(&cps, passed);

    let screen = passed.min(SCREENS.len() - 1);
    for (i, slug) in SCREENS[screen].iter().enumerate() {
        if i > 0 {
            println!();
        }
        print!("{}", render_content(repo_root, slug));
    }

    print_footer(workspace, &cps, passed);
    Ok(())
}

/// Render the progress panel: every checkpoint as ✓ (done) / → (current) /
/// ○ (upcoming). trace:STORY-34 | ai:claude
fn print_header(cps: &[Checkpoint; 5], passed: usize) {
    println!(
        "{}",
        "AIDA onboarding — the 15-minute shared-memory tour"
            .cyan()
            .bold()
    );
    println!();
    for (i, cp) in cps.iter().enumerate() {
        if i < passed {
            println!("  {} {}", "✓".green().bold(), cp.label.dimmed());
        } else if i == passed {
            println!(
                "  {} {}  {}",
                "→".cyan().bold(),
                cp.label.cyan().bold(),
                "← you are here".dimmed()
            );
        } else {
            println!("  {} {}", "○".dimmed(), cp.label.dimmed());
        }
    }
    println!();
    println!("{}", "─".repeat(64).dimmed());
    println!();
}

/// Render the footer: either the next-checkpoint nudge, or — once all five
/// are met — the finale. trace:STORY-34 | ai:claude
fn print_footer(workspace: &Path, cps: &[Checkpoint; 5], passed: usize) {
    println!();
    println!("{}", "─".repeat(64).dimmed());
    if passed >= cps.len() {
        println!(
            "{} The round trip is yours — all five checkpoints met.",
            "🎉".green()
        );
        println!(
            "  {}",
            "Browse the full 35-exercise track any time with `aida-tutor list`.".dimmed()
        );
        return;
    }
    let cp = &cps[passed];
    println!(
        "{} step {} of {} — {}",
        "→".cyan().bold(),
        passed + 1,
        SCREENS.len(),
        cp.label.bold()
    );
    println!("  {}", cp.nudge.dimmed());
    println!();
    println!(
        "  workspace: {}",
        workspace.display().to_string().cyan()
    );
    println!(
        "  {}",
        "Run `aida-tutor onboard` again once that checkpoint is met.".dimmed()
    );
}

/// Read a screen's markdown and render it for the terminal. Returns a
/// visible placeholder rather than erroring if the file is missing — a
/// missing content file shouldn't abort the whole tour.
fn render_content(repo_root: &Path, slug: &str) -> String {
    let path = repo_root.join(format!("content/onboarding/{slug}.md"));
    match std::fs::read_to_string(&path) {
        Ok(md) => crate::render_md_for_terminal(&md),
        Err(_) => format!("(onboarding content missing: {})\n", path.display()),
    }
}

/// Seed `workspace/` only if it isn't already the onboarding scratch
/// project. Refuses to clobber an in-progress exercise-track workspace —
/// the learner must opt in with `--reset`. trace:STORY-31 | ai:claude
fn ensure_seeded(workspace: &Path, repo_root: &Path) -> Result<()> {
    if workspace.join(SCRATCH_MARKER).exists() {
        return Ok(()); // already on the tour — keep the learner's state
    }
    if dir_has_entries(workspace) {
        bail!(
            "workspace/ already has work in it that isn't the onboarding tour.\n\
             Run `aida-tutor onboard --reset` to start the tour fresh — that wipes workspace/."
        );
    }
    seed(workspace, repo_root)?;
    println!(
        "{} seeded workspace/ with the `greet` scratch project.",
        "✓".green()
    );
    println!();
    Ok(())
}

/// True if `dir` exists and contains at least one entry.
fn dir_has_entries(dir: &Path) -> bool {
    std::fs::read_dir(dir)
        .map(|mut it| it.next().is_some())
        .unwrap_or(false)
}

/// Wipe `workspace/` and seed it fresh — the `--reset` path.
/// trace:STORY-31 | ai:claude
fn reseed(workspace: &Path, repo_root: &Path) -> Result<()> {
    if workspace.exists() {
        std::fs::remove_dir_all(workspace)
            .with_context(|| format!("removing {}", workspace.display()))?;
    }
    seed(workspace, repo_root)
}

/// Copy the scratch template into `workspace/` and make it a git repo with
/// one commit, so the learner can run `aida init` immediately.
///
/// Touches ONLY `workspace/` — never the repo-root progress file — which
/// is what keeps `--reset` from wiping a learner's exercise-track progress
/// (AC-5, `onboard_seed_is_reset_safe`). trace:STORY-31 | ai:claude
fn seed(workspace: &Path, repo_root: &Path) -> Result<()> {
    let template = repo_root.join("content/onboarding/scratch-template");
    if !template.is_dir() {
        bail!("scratch template missing at {}", template.display());
    }
    std::fs::create_dir_all(workspace)
        .with_context(|| format!("creating {}", workspace.display()))?;
    for entry in std::fs::read_dir(&template)
        .with_context(|| format!("reading {}", template.display()))?
    {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            std::fs::copy(entry.path(), workspace.join(entry.file_name()))
                .with_context(|| format!("seeding {:?}", entry.file_name()))?;
        }
    }
    // A git repo with one commit — the pre-state `aida init` needs.
    // `-b main` pins the default branch name so `aida show`'s git-linkage
    // section resolves cleanly at step 5's reveal (AIDA looks for `main`).
    git(workspace, &["init", "-q", "-b", "main"])?;
    git(workspace, &["config", "user.email", "learner@aida-tutor.local"])?;
    git(workspace, &["config", "user.name", "AIDA Learner"])?;
    git(workspace, &["add", "-A"])?;
    // `-c commit.gpgsign=false` keeps the seed commit working on machines
    // (and CI) that force-sign by default.
    git(
        workspace,
        &[
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-q",
            "-m",
            "Initial greet CLI",
        ],
    )?;
    Ok(())
}

/// Run `git -C <workspace> <args...>`, erroring on a non-zero exit.
fn git(workspace: &Path, args: &[&str]) -> Result<()> {
    let status = std::process::Command::new("git")
        .arg("-C")
        .arg(workspace)
        .args(args)
        .status()
        .with_context(|| format!("running `git {}`", args.join(" ")))?;
    if !status.success() {
        bail!("`git {}` failed in {}", args.join(" "), workspace.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A throwaway directory under the OS temp dir, unique per call.
    fn tmp() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "aida-tutor-onboard-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Write a minimal AIDA store into `ws` with one `FR-1` object at the
    /// given `status` — enough for the checkpoint predicates to inspect.
    fn store_with_fr(ws: &Path, status: &str) {
        std::fs::create_dir_all(ws.join(".aida")).unwrap();
        std::fs::write(ws.join(".aida/config.toml"), "").unwrap();
        std::fs::create_dir_all(ws.join(".aida-store/objects")).unwrap();
        std::fs::write(ws.join(".aida-store/metadata.yaml"), "").unwrap();
        std::fs::write(
            ws.join(".aida-store/objects/fr.yaml"),
            format!(
                "id: 019e-test-uuid\n\
                 spec_id: FR-1\n\
                 title: greet --upper flag\n\
                 status: {status}\n\
                 req_type: functional\n"
            ),
        )
        .unwrap();
    }

    fn run_git(ws: &Path, args: &[&str]) {
        let ok = std::process::Command::new("git")
            .arg("-C")
            .arg(ws)
            .args(args)
            .status()
            .unwrap()
            .success();
        assert!(ok, "git {args:?} failed");
    }

    #[test]
    fn onboard_verify_spec_created() {
        let ws = tmp();
        assert!(!cp_spec_captured(&ws), "no store yet → no spec");
        store_with_fr(&ws, "approved");
        assert!(cp_spec_captured(&ws), "FR-1 object present in the store");
        std::fs::remove_dir_all(&ws).ok();
    }

    #[test]
    fn onboard_verify_trace_comment_present() {
        let ws = tmp();
        store_with_fr(&ws, "approved");
        assert!(!cp_trace_present(&ws), "no trace comment yet");
        std::fs::write(
            ws.join("greet.py"),
            "# trace:FR-1 | ai:claude\nprint('hi')\n",
        )
        .unwrap();
        assert!(cp_trace_present(&ws), "greet.py carries trace:FR-1");
        std::fs::remove_dir_all(&ws).ok();
    }

    #[test]
    fn onboard_verify_commit_links_spec() {
        let ws = tmp();
        store_with_fr(&ws, "approved");
        run_git(&ws, &["init", "-q"]);
        run_git(&ws, &["config", "user.email", "t@t.local"]);
        run_git(&ws, &["config", "user.name", "T"]);
        std::fs::write(ws.join("greet.py"), "x\n").unwrap();
        run_git(&ws, &["add", "-A"]);
        run_git(&ws, &["-c", "commit.gpgsign=false", "commit", "-q", "-m", "Initial greet CLI"]);
        assert!(!cp_commit_links(&ws), "no commit names FR-1 yet");
        std::fs::write(ws.join("greet.py"), "y\n").unwrap();
        run_git(&ws, &["add", "-A"]);
        run_git(
            &ws,
            &[
                "-c",
                "commit.gpgsign=false",
                "commit",
                "-q",
                "-m",
                "[AI:claude] feat(greet): add --upper flag (FR-1)",
            ],
        );
        assert!(cp_commit_links(&ws), "commit subject references FR-1");
        std::fs::remove_dir_all(&ws).ok();
    }

    #[test]
    fn onboard_verify_status_flipped() {
        let ws = tmp();
        store_with_fr(&ws, "approved");
        assert!(!cp_status_flipped(&ws), "still approved → not flipped");
        std::fs::remove_dir_all(&ws).ok();

        let ws = tmp();
        store_with_fr(&ws, "completed");
        assert!(cp_status_flipped(&ws), "completed → flipped");
        std::fs::remove_dir_all(&ws).ok();
    }

    #[test]
    fn onboard_seed_is_reset_safe() {
        // seed + reseed must touch only workspace/, never the repo-root
        // progress file. trace:STORY-33 | ai:claude
        let root = tmp();
        let progress = root.join(".aida-tutor-progress.toml");
        std::fs::write(&progress, "completed = [1, 2, 3]\n").unwrap();
        let before = std::fs::read_to_string(&progress).unwrap();
        let ws = root.join("workspace");

        // Seed from the real template shipped in this crate.
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
        seed(&ws, repo_root).unwrap();
        assert!(ws.join(SCRATCH_MARKER).exists(), "scratch project seeded");
        reseed(&ws, repo_root).unwrap();
        assert!(ws.join(SCRATCH_MARKER).exists(), "scratch project re-seeded");

        assert_eq!(
            std::fs::read_to_string(&progress).unwrap(),
            before,
            "progress file untouched by seed/reseed"
        );
        std::fs::remove_dir_all(&root).ok();
    }
}
