//! aida-tutor — exercise-by-exercise walkthrough of the AIDA workflow.
//! trace:VIS-1, EPIC-1 | ai:claude

mod banner;
mod exercise;
mod exercises;
mod onboarding;
mod progress;
mod verify;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::{Path, PathBuf};

use crate::exercise::{Exercise, VerifyResult};
use crate::progress::Progress;

#[derive(Parser, Debug)]
#[command(name = "aida-tutor", version, about = "Hands-on tutor for AIDA")]
struct Cli {
    /// Banner logo geometry: `pyramid` (default) or the arch from shimmer.py.
    #[arg(long, global = true, value_enum)]
    logo: Option<banner::BannerLogo>,
    #[command(subcommand)]
    command: Option<Cmd>,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// List all exercises with their done/current/upcoming state.
    List,
    /// Show the current exercise (or one specified by id/slug).
    Show {
        /// Exercise id (1, 2, ...) or slug (init, vision, ...). Defaults
        /// to the current (lowest-incomplete) exercise.
        target: Option<String>,
    },
    /// Verify the current exercise against the workspace state.
    Verify {
        /// Override the current exercise to verify.
        target: Option<String>,
    },
    /// Show a hint for the current (or specified) exercise. Hints come in
    /// three depths: the bare `hint` is a one-paragraph nudge, `--more`
    /// gives concrete multi-step guidance, and `--solution` reveals the
    /// literal command as a last resort. trace:STORY-20 | ai:claude
    Hint {
        target: Option<String>,
        /// Multi-step nudge — concrete steps, stopping short of the
        /// literal command.
        #[arg(long)]
        more: bool,
        /// Reveal the literal command. Last resort: viewing it records
        /// the exercise as 'completed-with-solution' in your progress.
        #[arg(long, conflicts_with = "more")]
        solution: bool,
    },
    /// Wipe the workspace/ directory so you can start the current
    /// exercise fresh. Use with care — anything you've built is deleted.
    Reset {
        /// Skip the y/N prompt.
        #[arg(long, short = 'y')]
        yes: bool,
    },
    /// Print overall progress (e.g., "5/41 done — 12%").
    Progress,
    /// Re-run verify on the current exercise every time the workspace
    /// changes. Polls workspace/ mtime ~every 1.5s; Ctrl-C to exit.
    /// trace:STORY-18 | ai:claude
    Watch {
        /// Polling interval in milliseconds (default 1500).
        #[arg(long, default_value = "1500")]
        interval_ms: u64,
    },
    /// Non-interactive walkthrough: drive every exercise to a passing
    /// state, verifying each. Exits non-zero if any exercise fails. Run
    /// `reset --yes` first. Built for CI. trace:STORY-19 | ai:claude
    Demo,
    /// Install (or remove) a workspace-local `aida` wrapper that logs
    /// every invocation to `.aida-tutor-invocations.log`. Opt-in rigor:
    /// once the wrapper is on your PATH, the read-only exercises (3, 4,
    /// 10, 15, 17, 18) verify you actually ran the command, not just
    /// that the prerequisite state exists. trace:STORY-22 | ai:claude
    Wrapper {
        /// Remove the wrapper script + invocation log instead of
        /// installing.
        #[arg(long)]
        uninstall: bool,
    },
    /// Run the first-contact onboarding slice — a 15-minute guided tour of
    /// the AIDA round trip (capture → trace → commit → `aida show`), built
    /// only on the stable surface. A separate path from the 41-exercise
    /// track; self-paced, re-run it to advance. trace:EPIC-5 | ai:claude
    Onboard {
        /// Wipe workspace/ and restart the tour from step 1.
        #[arg(long)]
        reset: bool,
    },
    /// Print the next onboarding action without re-rendering the lesson.
    /// trace:STORY-52 | ai:codex
    Next,
    /// Start an interactive onboarding shell.
    /// trace:STORY-55 | ai:codex
    Shell,
    /// Start the legacy onboarding shell directly.
    #[command(name = "tutorial-shell", hide = true)]
    TutorialShell,
    /// Open the tutorial selector menu.
    /// trace:FR-9 | ai:codex
    Menu,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    if let Some(logo) = cli.logo {
        std::env::set_var("AIDA_TUTOR_LOGO", logo.as_str());
    }
    // Walk up from CWD to locate the aida-tutor repo root. This lets the
    // user run `aida-tutor` from inside `workspace/` (the natural place
    // when working on an exercise) without having to `cd ..` first.
    let cwd = std::env::current_dir()?;
    let repo_root = find_tutor_root(&cwd).unwrap_or_else(|| cwd.clone());
    let workspace = repo_root.join("workspace");
    let exercises = exercises::all();
    let mut prog = Progress::load(&repo_root)?;

    match cli.command.unwrap_or(Cmd::Menu) {
        Cmd::List => cmd_list(&exercises, &workspace, &prog),
        Cmd::Show { target } => {
            cmd_show(&exercises, &workspace, &repo_root, &prog, target.as_deref())
        }
        Cmd::Verify { target } => cmd_verify(
            &exercises,
            &workspace,
            &repo_root,
            &mut prog,
            target.as_deref(),
        ),
        Cmd::Hint {
            target,
            more,
            solution,
        } => cmd_hint(
            &exercises,
            &repo_root,
            &mut prog,
            target.as_deref(),
            more,
            solution,
        ),
        Cmd::Reset { yes } => cmd_reset(&workspace, yes),
        Cmd::Progress => cmd_progress(&exercises, &prog),
        Cmd::Watch { interval_ms } => {
            cmd_watch(&exercises, &workspace, &repo_root, &mut prog, interval_ms)
        }
        Cmd::Demo => cmd_demo(&exercises, &workspace),
        Cmd::Wrapper { uninstall } => cmd_wrapper(&workspace, uninstall),
        Cmd::Onboard { reset } => onboarding::run(&workspace, &repo_root, reset),
        Cmd::Next => onboarding::next(&workspace, &repo_root),
        Cmd::Shell => onboarding::aida_shell(&cwd, &repo_root),
        Cmd::TutorialShell => onboarding::shell(&workspace, &repo_root),
        Cmd::Menu => onboarding::menu(&cwd, &repo_root),
    }
}

/// Walk every exercise non-interactively: run its `demo` solution, then
/// `verify`. Exits non-zero on the first batch of non-passing exercises.
/// Does NOT touch `.aida-tutor-progress.toml` — demo is a CI gate, not a
/// way to earn completion credit. trace:STORY-19 | ai:claude
fn cmd_demo(exercises: &[Box<dyn Exercise>], workspace: &Path) -> Result<()> {
    if !workspace.exists() {
        anyhow::bail!("workspace/ doesn't exist — run `aida-tutor reset --yes` before `demo`");
    }
    println!("{}", "aida-tutor demo — non-interactive walkthrough".bold());
    println!();
    // A real learner works the exercises interactively, and AIDA grants an
    // interactive session the authority to approve specs, mark them done,
    // and route work to queues. The demo is the non-interactive CI proxy
    // for that learner, so it carries the equivalent advisor authority from
    // the start. Individual exercise demos may switch roles (e.g. the
    // producer/consumer queue arc). trace:STORY-46 | ai:claude
    std::env::set_var("AIDA_SESSION_ROLE", "advisor");
    let mut failures = 0u32;
    for ex in exercises {
        match ex.demo(workspace).map(|()| ex.verify(workspace)) {
            Ok(VerifyResult::Pass) => {
                println!("  {:>2} {:<22} {}", ex.id(), ex.slug(), "pass".green());
            }
            Ok(VerifyResult::Pending(msg)) => {
                println!(
                    "  {:>2} {:<22} {} pending: {}",
                    ex.id(),
                    ex.slug(),
                    "✗".red().bold(),
                    msg
                );
                failures += 1;
            }
            Ok(VerifyResult::Fail(msg)) => {
                println!(
                    "  {:>2} {:<22} {} fail: {}",
                    ex.id(),
                    ex.slug(),
                    "✗".red().bold(),
                    msg
                );
                failures += 1;
            }
            Err(e) => {
                println!(
                    "  {:>2} {:<22} {} demo step errored:\n{}",
                    ex.id(),
                    ex.slug(),
                    "✗".red().bold(),
                    e
                );
                failures += 1;
            }
        }
    }
    println!();
    if failures > 0 {
        anyhow::bail!("{} of {} exercises did not pass", failures, exercises.len());
    }
    println!(
        "{} all {} exercises passed",
        "✓".green().bold(),
        exercises.len()
    );
    Ok(())
}

fn cmd_watch(
    exercises: &[Box<dyn Exercise>],
    workspace: &Path,
    repo_root: &Path,
    prog: &mut Progress,
    interval_ms: u64,
) -> Result<()> {
    use std::time::Duration;

    println!(
        "{} {} (Ctrl-C to exit)",
        "Watching".cyan().bold(),
        workspace.display()
    );
    println!();

    let mut last_signature: Option<u64> = None;
    let mut last_current: Option<u32> = None;

    loop {
        // Cheap "did anything change?" signature: max mtime + total file
        // count under workspace/. trace:STORY-18 | ai:claude
        let sig = workspace_signature(workspace);
        let total = exercises.len() as u32;
        let current_id = prog.current(total);

        let need_render = last_signature != Some(sig) || last_current != current_id;
        if need_render {
            // Clear screen + jump home — terminal-ish but not too aggressive.
            print!("\x1B[2J\x1B[H");
            println!(
                "{} {} (Ctrl-C to exit)",
                "Watching".cyan().bold(),
                workspace.display()
            );
            println!(
                "  refresh every {:.1}s — last tick: {}",
                interval_ms as f64 / 1000.0,
                chrono_now_local()
            );
            println!();

            match current_id {
                None => {
                    println!("{} all {} exercises complete.", "🎉".green().bold(), total);
                }
                Some(id) => {
                    if let Some(ex) = exercises.iter().find(|e| e.id() == id) {
                        println!(
                            "{} {:02} — {}",
                            "Current:".bold(),
                            ex.id(),
                            ex.title().cyan()
                        );
                        let result = ex.verify(workspace);
                        match result {
                            VerifyResult::Pass => {
                                if !prog.is_completed(ex.id()) {
                                    prog.record_completion(ex.id());
                                    let _ = prog.save(repo_root);
                                }
                                println!(
                                    "  {} on-disk state matches expectation — recorded.",
                                    "✓".green().bold()
                                );
                            }
                            VerifyResult::Pending(msg) => {
                                println!("  {} {}", "·".dimmed(), msg.dimmed());
                                println!(
                                    "  {}",
                                    "(`aida-tutor hint` for a nudge in another shell)".dimmed()
                                );
                            }
                            VerifyResult::Fail(msg) => {
                                println!("  {} {}", "✗".red().bold(), msg);
                            }
                        }
                    }
                }
            }
            last_signature = Some(sig);
            last_current = current_id;
        }

        // Sleep before next poll. 1.5s is fine for "do something, see
        // result" responsiveness without burning CPU on the file walk.
        std::thread::sleep(Duration::from_millis(interval_ms));
    }
}

/// Cheap signature: hash of (max mtime nanos × file count). Picks up any
/// add/edit/delete under workspace/ without needing inotify dependencies.
fn workspace_signature(workspace: &std::path::Path) -> u64 {
    use std::time::SystemTime;
    let mut max_mtime: u64 = 0;
    let mut count: u64 = 0;
    for entry in walkdir::WalkDir::new(workspace)
        .into_iter()
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        count += 1;
        if let Ok(meta) = entry.metadata() {
            if let Ok(m) = meta.modified() {
                if let Ok(d) = m.duration_since(SystemTime::UNIX_EPOCH) {
                    let n = d.as_secs() * 1_000_000_000 + d.subsec_nanos() as u64;
                    if n > max_mtime {
                        max_mtime = n;
                    }
                }
            }
        }
    }
    max_mtime.wrapping_mul(31).wrapping_add(count)
}

fn chrono_now_local() -> String {
    // Local time per Joe's pref (memory: feedback_local_time).
    // trace:feedback_local_time | ai:claude
    chrono::Local::now().format("%H:%M:%S").to_string()
}

fn pick<'a>(
    exercises: &'a [Box<dyn Exercise>],
    target: Option<&str>,
    prog: &Progress,
) -> Option<&'a Box<dyn Exercise>> {
    if let Some(t) = target {
        if let Ok(id) = t.parse::<u32>() {
            return exercises.iter().find(|e| e.id() == id);
        }
        return exercises.iter().find(|e| e.slug() == t);
    }
    let total = exercises.len() as u32;
    let cur_id = prog.current(total).unwrap_or(total);
    exercises.iter().find(|e| e.id() == cur_id)
}

/// Last exercise of the core "15-minute novice loop" (capture → build →
/// link → done). Exercises past this are the optional "Going further"
/// track. trace:STORY-46 | ai:claude
const CORE_LAST: u32 = 8;

fn cmd_list(exercises: &[Box<dyn Exercise>], workspace: &Path, prog: &Progress) -> Result<()> {
    let total = exercises.len() as u32;
    let current = prog.current(total).unwrap_or(0);
    println!("{}", format!("aida-tutor — {} exercises", total).bold());
    println!();
    println!("{}", "  Core — the 15-minute loop".cyan().bold());
    for e in exercises {
        if e.id() == CORE_LAST + 1 {
            println!();
            println!(
                "{}",
                "  Going further (optional) — pick what's useful, in any order"
                    .cyan()
                    .bold()
            );
        }
        let marker = if prog.is_completed(e.id()) {
            "✓".green().to_string()
        } else if e.id() == current {
            "→".cyan().bold().to_string()
        } else {
            " ".to_string()
        };
        // Inline live-verify state to give the user a quick "where am I"
        // snapshot without running `verify` per row by hand.
        let state = if prog.is_completed(e.id()) {
            // 'completed-with-solution' marks a finish that leaned on the
            // `hint --solution` escape valve. trace:STORY-20 | ai:claude
            if prog.used_solution(e.id()) {
                "completed-with-solution".yellow().to_string()
            } else {
                "completed".green().to_string()
            }
        } else {
            match e.verify(workspace) {
                VerifyResult::Pass => "passes — run `aida-tutor verify <number>` to record"
                    .yellow()
                    .to_string(),
                VerifyResult::Pending(_) => "pending".dimmed().to_string(),
                VerifyResult::Fail(_) => "needs fix".red().to_string(),
            }
        };
        println!("{} {:>2}. {:<48} [{}]", marker, e.id(), e.title(), state);
    }
    println!();
    println!(
        "{}",
        "To continue: `aida-tutor show <number>` opens an exercise; `aida-tutor verify <number>` records a pass."
            .dimmed()
    );
    Ok(())
}

fn cmd_show(
    exercises: &[Box<dyn Exercise>],
    workspace: &Path,
    repo_root: &Path,
    prog: &Progress,
    target: Option<&str>,
) -> Result<()> {
    let Some(ex) = pick(exercises, target, prog) else {
        anyhow::bail!("no such exercise");
    };
    println!(
        "{}",
        format!("Exercise {:02} — {}", ex.id(), ex.title())
            .cyan()
            .bold()
    );
    println!("{}", "─".repeat(60).dimmed());
    println!();
    println!("{}", render_md_for_terminal(&ex.description(repo_root)));
    println!();
    println!("{}", "─".repeat(60).dimmed());
    let state = ex.verify(workspace);
    match state {
        VerifyResult::Pass => {
            if prog.is_completed(ex.id()) {
                println!("{} already completed.", "✓".green());
            } else {
                println!(
                    "{} on-disk state matches expectation. Run `aida-tutor verify` to record completion.",
                    "✓".green()
                );
            }
        }
        VerifyResult::Pending(msg) => {
            println!("{} pending: {}", "·".dimmed(), msg.dimmed());
            println!(
                "{}",
                "Run `aida-tutor hint` for a nudge — `--more` for steps, `--solution` for the command."
                    .dimmed()
            );
        }
        VerifyResult::Fail(msg) => {
            println!("{} {}", "✗".red(), msg);
        }
    }
    println!();
    println!("Workspace: {}", workspace.display().to_string().cyan());
    Ok(())
}

fn cmd_verify(
    exercises: &[Box<dyn Exercise>],
    workspace: &Path,
    repo_root: &Path,
    prog: &mut Progress,
    target: Option<&str>,
) -> Result<()> {
    let Some(ex) = pick(exercises, target, prog) else {
        anyhow::bail!("no such exercise");
    };
    match ex.verify(workspace) {
        VerifyResult::Pass => {
            if prog.is_completed(ex.id()) {
                println!("{} exercise {:02} already complete.", "✓".green(), ex.id());
            } else {
                prog.record_completion(ex.id());
                // Capture the distributed-store baseline at the boundary
                // between exercises 19 and 20. A fixed bootstrap count is
                // coupled to aida init internals and can false-pass when
                // that bootstrap history changes. trace:TASK-3 | ai:codex
                if ex.id() == 19 && prog.store_commit_baseline.is_none() {
                    prog.store_commit_baseline =
                        crate::verify::git_commit_count(workspace, "aida-store");
                }
                // Save progress at the *repo root* (resolved by
                // find_tutor_root), not CWD — otherwise running verify
                // from inside workspace/ writes the progress file to
                // workspace/, where the next `reset` wipes it.
                prog.save(repo_root)?;
                println!(
                    "{} exercise {:02} — {}",
                    "✓".green().bold(),
                    ex.id(),
                    ex.title()
                );
                // Finishing the core loop is its own milestone — call it
                // out, and frame the rest as optional. trace:STORY-46
                if ex.id() == CORE_LAST {
                    println!();
                    println!(
                        "{}",
                        "🎉 Core loop complete — you captured a spec, built it, linked the \
                         code, and closed it out."
                            .green()
                            .bold()
                    );
                    println!(
                        "{}",
                        "Everything past here is optional 'Going further'. Take it in any \
                         order, or stop here — you've got the loop."
                            .dimmed()
                    );
                    println!();
                }
                let total = exercises.len() as u32;
                if let Some(next_id) = prog.current(total) {
                    if let Some(next_ex) = exercises.iter().find(|e| e.id() == next_id) {
                        println!("Next: {:02} — {}", next_ex.id(), next_ex.title().cyan());
                        println!("Run `aida-tutor show` to see the next exercise.");
                    }
                } else {
                    println!();
                    println!(
                        "{}",
                        "🎉 All exercises complete. You've walked the full AIDA loop."
                            .green()
                            .bold()
                    );
                }
            }
        }
        VerifyResult::Pending(msg) => {
            println!("{} pending — {}", "·".dimmed(), msg);
        }
        VerifyResult::Fail(msg) => {
            println!("{} {}", "✗".red().bold(), msg);
        }
    }
    Ok(())
}

/// Render a hint at one of three depths for the current (or targeted)
/// exercise. Plain `hint` is a one-paragraph nudge; `--more` surfaces the
/// per-exercise multi-step guidance; `--solution` reveals the literal
/// command and stamps the exercise `completed-with-solution` in progress.
/// trace:STORY-20 | ai:claude
fn cmd_hint(
    exercises: &[Box<dyn Exercise>],
    repo_root: &Path,
    prog: &mut Progress,
    target: Option<&str>,
    more: bool,
    solution: bool,
) -> Result<()> {
    let Some(ex) = pick(exercises, target, prog) else {
        anyhow::bail!("no such exercise");
    };
    let id = ex.id();
    println!("{} {:02} — {}", "Hint for exercise".bold(), id, ex.title());
    println!();

    if solution {
        // Level 3 — the literal command, the deepest escape valve.
        match ex.hint_solution() {
            Some(cmd) => {
                println!(
                    "{}",
                    "Solution — the literal command (last resort):"
                        .yellow()
                        .bold()
                );
                println!();
                for line in cmd.lines() {
                    println!("    {}", line.cyan().bold());
                }
                // Stamp solution use so completion records as
                // 'completed-with-solution' — but only while the exercise
                // is still open. Peeking after a clean finish is harmless
                // review and shouldn't tarnish the record.
                if !prog.is_completed(id) {
                    if !prog.used_solution(id) {
                        prog.record_solution_used(id);
                        prog.save(repo_root)?;
                    }
                    println!();
                    println!(
                        "{}",
                        "(noted — this exercise will show as 'completed-with-solution')".dimmed()
                    );
                }
            }
            None => {
                println!(
                    "{}",
                    "No literal solution is recorded for this exercise.".dimmed()
                );
                println!();
                println!("{}", ex.hint_more().unwrap_or_else(|| ex.hint()));
            }
        }
    } else if more {
        // Level 2 — concrete multi-step guidance.
        match ex.hint_more() {
            Some(steps) => {
                println!("{}", steps);
                println!();
                println!(
                    "{}",
                    "Still stuck? `aida-tutor hint --solution` shows the literal command.".dimmed()
                );
            }
            None => {
                println!("{}", ex.hint());
                println!();
                println!(
                    "{}",
                    "(no extra steps for this one — `aida-tutor hint --solution` for the command)"
                        .dimmed()
                );
            }
        }
    } else {
        // Level 1 — the one-paragraph nudge.
        println!("{}", ex.hint());
        println!();
        println!(
            "{}",
            "Stuck? `aida-tutor hint --more` for steps, `--solution` for the command.".dimmed()
        );
    }
    Ok(())
}

/// Install or remove the optional invocation-logging `aida` wrapper.
///
/// The wrapper is a tiny `/bin/sh` script at `workspace/.aida-tutor-bin/
/// aida` that appends every call to `.aida-tutor-invocations.log` and
/// then exec's the real `aida`. Putting `.aida-tutor-bin/` first on
/// `PATH` routes `aida` through it; the read-only exercises (3, 4, 10,
/// 15, 17, 18) then verify the command actually ran instead of passing
/// on prerequisite state. Opt-in, and wiped by any `reset`.
/// trace:STORY-22 | ai:claude
fn cmd_wrapper(workspace: &Path, uninstall: bool) -> Result<()> {
    if !workspace.exists() {
        anyhow::bail!(
            "workspace/ doesn't exist — run `aida-tutor reset --yes` before installing the wrapper"
        );
    }
    let bin_dir = workspace.join(".aida-tutor-bin");
    let script = bin_dir.join("aida");
    let log = workspace.join(".aida-tutor-invocations.log");

    if uninstall {
        let was_installed = bin_dir.exists() || log.exists();
        let _ = std::fs::remove_dir_all(&bin_dir);
        let _ = std::fs::remove_file(&log);
        if was_installed {
            println!(
                "{} wrapper removed — verifiers fall back to prerequisite-state checks.",
                "✓".green()
            );
        } else {
            println!(
                "{} no wrapper was installed — nothing to remove.",
                "·".dimmed()
            );
        }
        return Ok(());
    }

    // Resolve the real `aida` so the wrapper exec's the CLI, never
    // itself. Skip our own script if a previous install left it on PATH.
    let real_aida =
        find_real_aida(&script).context("locating a real `aida` on PATH — install AIDA first")?;

    std::fs::create_dir_all(&bin_dir).with_context(|| format!("creating {}", bin_dir.display()))?;
    // Note: the script body deliberately carries no `trace:` token —
    // it lives under workspace/ and would otherwise be picked up by the
    // exercise-10 trace-comment scan.
    let script_body = format!(
        "#!/bin/sh\n\
         # aida-tutor invocation-logging wrapper.\n\
         # Generated by `aida-tutor wrapper`: logs every call to the file\n\
         # below, then hands off to the real AIDA CLI. Remove it with\n\
         # `aida-tutor wrapper --uninstall` (any `aida-tutor reset` wipes it too).\n\
         printf '%s\\t%s\\n' \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\" \"$*\" >> {log}\n\
         exec {real} \"$@\"\n",
        log = sh_single_quote(&log.to_string_lossy()),
        real = sh_single_quote(&real_aida.to_string_lossy()),
    );
    std::fs::write(&script, &script_body)
        .with_context(|| format!("writing {}", script.display()))?;
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755))
            .with_context(|| format!("setting +x on {}", script.display()))?;
    }
    // Arm the opt-in immediately: an empty log makes the read-only
    // verifiers expect a real invocation from this point on.
    if !log.exists() {
        std::fs::write(&log, "").with_context(|| format!("creating {}", log.display()))?;
    }

    let abs_bin = bin_dir.canonicalize().unwrap_or_else(|_| bin_dir.clone());
    println!(
        "{} invocation-logging wrapper installed.",
        "✓".green().bold()
    );
    println!();
    println!("  shim       {}", script.display().to_string().cyan());
    println!("  real aida  {}", real_aida.display().to_string().dimmed());
    println!("  log        {}", log.display().to_string().cyan());
    println!();
    println!("Put it first on your PATH so `aida` routes through it:");
    println!();
    println!(
        "    {}",
        format!("export PATH=\"{}:$PATH\"", abs_bin.display())
            .cyan()
            .bold()
    );
    println!();
    println!("Every `aida ...` call then appends to the log. With the wrapper active,");
    println!(
        "exercises 7, 8, 13, 15, 16 and 17 verify you actually {} the command —",
        "ran".bold()
    );
    println!("not just that the prerequisite state exists.");
    println!();
    println!(
        "{}",
        "Turn it back off with `aida-tutor wrapper --uninstall`.".dimmed()
    );
    Ok(())
}

/// Scan `PATH` for a real `aida` executable, skipping `skip` — our own
/// wrapper script, which may already be on `PATH` from a prior install.
/// trace:STORY-22 | ai:claude
fn find_real_aida(skip: &Path) -> Result<PathBuf> {
    let skip_canon = skip.canonicalize().ok();
    let path = std::env::var_os("PATH").context("PATH is not set")?;
    for dir in std::env::split_paths(&path) {
        let cand = dir.join("aida");
        if skip_canon.is_some() && cand.canonicalize().ok() == skip_canon {
            continue; // this is our own wrapper — keep looking
        }
        if is_executable_file(&cand) {
            return Ok(cand);
        }
    }
    anyhow::bail!("no `aida` executable found on PATH")
}

/// True if `p` is a regular file with at least one executable bit set.
/// trace:STORY-22 | ai:claude
fn is_executable_file(p: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(p)
        .map(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

/// Wrap `s` in single quotes for safe embedding in a `/bin/sh` script,
/// escaping any embedded single quote (`'` → `'\''`).
/// trace:STORY-22 | ai:claude
fn sh_single_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

fn cmd_reset(workspace: &Path, yes: bool) -> Result<()> {
    if !yes && workspace.exists() {
        use std::io::{self, Write};
        print!(
            "About to delete {} — are you sure? [y/N] ",
            workspace.display()
        );
        io::stdout().flush()?;
        let mut answer = String::new();
        io::stdin().read_line(&mut answer)?;
        if !matches!(answer.trim().to_lowercase().as_str(), "y" | "yes") {
            println!("Aborted.");
            return Ok(());
        }
    }
    if workspace.exists() {
        std::fs::remove_dir_all(workspace)
            .with_context(|| format!("removing {}", workspace.display()))?;
    }
    // The session-worktree exercises (27-30) leave `aida session start`
    // worktrees as `workspace-<slug>/` siblings. Wiping workspace/ alone
    // orphans them (their `.git` pointer dangles), so reset clears them
    // too — a clean slate means no stale worktrees. trace:STORY-28
    if let Some(parent) = workspace.parent() {
        if let Ok(read_dir) = std::fs::read_dir(parent) {
            for entry in read_dir.filter_map(Result::ok) {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if name.starts_with("workspace-") && entry.path().is_dir() {
                    let _ = std::fs::remove_dir_all(entry.path());
                }
            }
        }
    }
    std::fs::create_dir_all(workspace)?;
    // Initialize a bare git repo inside workspace/ so the user can run
    // `aida init` (which requires a git repo). This is the ONLY pre-state
    // the tutor sets up; the rest is the user's job.
    let res = std::process::Command::new("git")
        .arg("-C")
        .arg(workspace)
        .args(["init", "-q"])
        .status()?;
    if !res.success() {
        anyhow::bail!("git init in workspace/ failed");
    }
    // Configure identity so first commits work without further setup.
    let _ = std::process::Command::new("git")
        .arg("-C")
        .arg(workspace)
        .args(["config", "user.email", "learner@aida-tutor.local"])
        .status();
    let _ = std::process::Command::new("git")
        .arg("-C")
        .arg(workspace)
        .args(["config", "user.name", "AIDA Learner"])
        .status();
    // Seed a README so there's something to commit after the first init.
    let readme = workspace.join("README.md");
    std::fs::write(&readme, "# my-project\n")?;
    let _ = std::process::Command::new("git")
        .arg("-C")
        .arg(workspace)
        .args(["add", "README.md"])
        .status();
    let _ = std::process::Command::new("git")
        .arg("-C")
        .arg(workspace)
        .args(["commit", "-q", "-m", "init"])
        .status();
    println!(
        "{} workspace reset — {} is a fresh git repo.",
        "✓".green(),
        workspace.display()
    );
    Ok(())
}

/// Walk upward from `start` looking for the aida-tutor repo root. The
/// marker is the presence of both `content/01-init.md` (tutor-specific)
/// AND `Cargo.toml` (so we don't accidentally match a sibling project
/// that happens to have a `content/` dir). Returns None if no ancestor
/// matches — caller should fall back to CWD.
// trace:FR-19,FR-20 | ai:antigravity
pub(crate) fn find_tutor_root(start: &Path) -> Option<PathBuf> {
    if let Ok(env_root) = std::env::var("AIDA_TUTOR_ROOT") {
        let p = PathBuf::from(env_root);
        if p.join("content/01-init.md").exists() && p.join("Cargo.toml").exists() {
            return Some(p);
        }
    }
    let mut cur = Some(start.to_path_buf());
    while let Some(dir) = cur {
        if dir.join("content/01-init.md").exists() && dir.join("Cargo.toml").exists() {
            return Some(dir);
        }
        cur = dir.parent().map(|p| p.to_path_buf());
    }
    if let Ok(exe) = std::env::current_exe() {
        let mut cur = exe.parent().map(|p| p.to_path_buf());
        while let Some(dir) = cur {
            if dir.join("content/01-init.md").exists() && dir.join("Cargo.toml").exists() {
                return Some(dir);
            }
            cur = dir.parent().map(|p| p.to_path_buf());
        }
    }
    None
}

fn detect_terminal_width() -> Option<usize> {
    use std::io::IsTerminal;
    if !std::io::stdout().is_terminal() {
        return None;
    }
    #[cfg(unix)]
    unsafe {
        let mut winsize: libc::winsize = std::mem::zeroed();
        if libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut winsize) == 0 && winsize.ws_col > 0 {
            return Some(winsize.ws_col as usize);
        }
    }
    std::env::var("COLUMNS")
        .ok()
        .and_then(|c| c.parse::<usize>().ok())
}

// trace:FR-22 | ai:antigravity
pub(crate) fn target_terminal_width() -> usize {
    detect_terminal_width()
        .map(|w| (w.saturating_sub(2)).clamp(40, 78))
        .unwrap_or(78)
}

// trace:FR-22 | ai:antigravity
fn visible_len(s: &str) -> usize {
    let mut len = 0;
    let mut in_escape = false;
    for ch in s.chars() {
        if ch == '\x1b' {
            in_escape = true;
        } else if in_escape {
            if ch == 'm' {
                in_escape = false;
            }
        } else {
            len += 1;
        }
    }
    len
}

// trace:FR-22 | ai:antigravity
fn style_inline(text: &str) -> String {
    static INLINE_CODE_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    static BOLD_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    static ITALIC_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();

    let code_re = INLINE_CODE_RE.get_or_init(|| regex::Regex::new(r"`([^`]+)`").unwrap());
    let bold_re = BOLD_RE.get_or_init(|| regex::Regex::new(r"\*\*([^*]+)\*\*").unwrap());
    let italic_re = ITALIC_RE.get_or_init(|| regex::Regex::new(r"(?:\*|_)([^*_]+)(?:\*|_)").unwrap());

    // 1. Inline code: `...` -> cyan per word
    let s = code_re.replace_all(text, |caps: &regex::Captures| {
        caps[1]
            .split(' ')
            .map(|w| w.cyan().to_string())
            .collect::<Vec<_>>()
            .join(" ")
    });
    // 2. Bold: **...** -> bold per word
    let s = bold_re.replace_all(&s, |caps: &regex::Captures| {
        caps[1]
            .split(' ')
            .map(|w| w.bold().to_string())
            .collect::<Vec<_>>()
            .join(" ")
    });
    // 3. Italic: *...* or _..._ -> italic per word
    let s = italic_re.replace_all(&s, |caps: &regex::Captures| {
        caps[1]
            .split(' ')
            .map(|w| w.italic().to_string())
            .collect::<Vec<_>>()
            .join(" ")
    });
    s.to_string()
}

// trace:FR-22 | ai:antigravity
fn wrap_words(text: &str, max_width: usize, first_prefix: &str, rest_prefix: &str) -> String {
    let mut out = String::new();
    let mut current_line = String::new();
    let mut current_len = 0;
    let mut prefix = first_prefix;

    for word in text.split_whitespace() {
        let w_len = visible_len(word);
        if current_len == 0 {
            current_line.push_str(prefix);
            current_line.push_str(word);
            current_len = visible_len(prefix) + w_len;
        } else if current_len + 1 + w_len <= max_width {
            current_line.push(' ');
            current_line.push_str(word);
            current_len += 1 + w_len;
        } else {
            out.push_str(&current_line);
            out.push('\n');
            current_line.clear();
            prefix = rest_prefix;
            current_line.push_str(prefix);
            current_line.push_str(word);
            current_len = visible_len(prefix) + w_len;
        }
    }
    if !current_line.is_empty() {
        out.push_str(&current_line);
        out.push('\n');
    }
    out
}

fn flush_prose(out: &mut String, pending: &mut Vec<String>, max_width: usize) {
    if pending.is_empty() {
        return;
    }
    let p_text = pending.join(" ");
    pending.clear();
    let styled = style_inline(&p_text);
    let wrapped = wrap_words(&styled, max_width, "", "");
    out.push_str(&wrapped);
    out.push('\n');
}

/// Markdown-to-terminal renderer with word wrapping and clean styling.
// trace:EPIC-5,FR-22 | ai:codex,antigravity
pub(crate) fn render_md_for_terminal_with_width(md: &str, max_width: usize) -> String {
    let mut out = String::new();
    let mut in_fence = false;
    let mut in_html_comment = false;
    let mut pending_prose: Vec<String> = Vec::new();

    for line in md.lines() {
        let t = line.trim();

        if in_fence {
            if t.starts_with("```") {
                in_fence = false;
                if !out.ends_with("\n\n") {
                    out.push('\n');
                }
            } else {
                out.push_str(&format!("    {}\n", line.cyan().bold()));
            }
            continue;
        }

        if in_html_comment {
            if t.contains("-->") {
                in_html_comment = false;
            }
            continue;
        }

        if t.starts_with("<!--") {
            if !t.contains("-->") {
                in_html_comment = true;
            }
            continue;
        }

        if t.starts_with("```") {
            flush_prose(&mut out, &mut pending_prose, max_width);
            in_fence = true;
            continue;
        }

        if t.is_empty() {
            flush_prose(&mut out, &mut pending_prose, max_width);
            continue;
        }

        if t.starts_with('#') {
            flush_prose(&mut out, &mut pending_prose, max_width);
            let level = t.chars().take_while(|c| *c == '#').count();
            let htext = t[level..].trim();
            if !out.is_empty() && !out.ends_with("\n\n") {
                out.push('\n');
            }
            if level <= 2 {
                out.push_str(&format!("{}\n\n", htext.cyan().bold()));
            } else {
                out.push_str(&format!("{}\n\n", htext.bold()));
            }
            continue;
        }

        if t == "---" || t == "***" || (t.starts_with("───") && t.chars().all(|c| c == '─')) {
            flush_prose(&mut out, &mut pending_prose, max_width);
            if !out.is_empty() && !out.ends_with("\n\n") {
                out.push('\n');
            }
            let rule_len = max_width.min(78);
            out.push_str(&format!("{}\n\n", "─".repeat(rule_len).dimmed()));
            continue;
        }

        if t.starts_with("- ") || t.starts_with("* ") || t.starts_with("+ ") {
            flush_prose(&mut out, &mut pending_prose, max_width);
            let item_text = style_inline(t[2..].trim());
            out.push_str(&wrap_words(&item_text, max_width, "  • ", "    "));
            continue;
        }

        let digit_count = t.chars().take_while(|c| c.is_ascii_digit()).count();
        if digit_count > 0 && t[digit_count..].starts_with(". ") {
            flush_prose(&mut out, &mut pending_prose, max_width);
            let num_str = &t[..digit_count + 1];
            let prefix = format!("  {num_str} ");
            let hang = " ".repeat(prefix.len());
            let item_text = style_inline(t[digit_count + 2..].trim());
            out.push_str(&wrap_words(&item_text, max_width, &prefix, &hang));
            continue;
        }

        if line.starts_with("    ")
            || line.starts_with('\t')
            || line.starts_with("  →")
            || line.starts_with("  ○")
            || line.starts_with("  ✓")
        {
            flush_prose(&mut out, &mut pending_prose, max_width);
            out.push_str(&style_inline(line));
            out.push('\n');
            continue;
        }

        pending_prose.push(t.to_string());
    }

    flush_prose(&mut out, &mut pending_prose, max_width);
    out
}

// trace:EPIC-5,FR-22 | ai:codex,antigravity
pub(crate) fn render_md_for_terminal(md: &str) -> String {
    render_md_for_terminal_with_width(md, target_terminal_width())
}

fn cmd_progress(exercises: &[Box<dyn Exercise>], prog: &Progress) -> Result<()> {
    let total = exercises.len();
    let done = prog.completed.len();
    let pct = if total == 0 { 0 } else { (done * 100) / total };
    println!("{} / {} exercises complete ({}%)", done, total, pct);
    // Surface how many completions leaned on the `--solution` escape
    // valve — 'completed-with-solution' vs a clean 'completed'.
    // trace:STORY-20 | ai:claude
    let with_solution = prog
        .completed
        .iter()
        .filter(|id| prog.used_solution(**id))
        .count();
    if with_solution > 0 {
        println!(
            "  {} of those completed-with-solution (used `aida-tutor hint --solution`)",
            with_solution
        );
    }
    Ok(())
}

#[allow(dead_code)]
fn pretty_pathbuf(p: &PathBuf) -> String {
    p.display().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    // trace:FR-22 | ai:antigravity
    #[test]
    fn render_md_strips_html_comments() {
        let md = "<!-- trace:STORY-47,STORY-48 | ai:codex -->\n## Step 1\nHello world";
        let rendered = render_md_for_terminal_with_width(md, 78);
        assert!(!rendered.contains("trace:STORY-47"));
        assert!(!rendered.contains("<!--"));
        assert!(rendered.contains("Step 1"));
        assert!(rendered.contains("Hello world"));
    }

    // trace:FR-22 | ai:antigravity
    #[test]
    fn render_md_formats_headings_without_octothorpes() {
        let md = "## Step 1 — give the project a memory\nParagraph text";
        let rendered = render_md_for_terminal_with_width(md, 78);
        assert!(!rendered.contains("##"));
        assert!(rendered.contains("Step 1 — give the project a memory"));
    }

    // trace:FR-22 | ai:antigravity
    #[test]
    fn render_md_wraps_long_paragraphs_cleanly() {
        let md = "This uses AIDA's Codex init profile and does not scaffold Claude Code. Current AIDA 0.14 may still write .antigravity/ even though Antigravity was not selected; strict all-agent allow-list behavior is tracked upstream as BUG-794. If .antigravity/ appears, remove it before continuing this compliance-safe tour.";
        let rendered = render_md_for_terminal_with_width(md, 78);
        for line in rendered.lines() {
            assert!(
                visible_len(line) <= 78,
                "Line exceeded 78 columns: visible_len={}, line={}",
                visible_len(line),
                line
            );
        }
        // Word "behavior" should remain intact, not split into "b \n ehavior"
        assert!(rendered.contains("behavior"));
        assert!(!rendered.contains("\nb\n") && !rendered.contains("b\nehavior"));
    }

    // trace:FR-22 | ai:antigravity
    #[test]
    fn render_md_preserves_fenced_code_blocks() {
        let md = "Before code:\n```\ncd workspace\naida init --agent codex\n```\nAfter code";
        let rendered = render_md_for_terminal_with_width(md, 78);
        assert!(rendered.contains("cd workspace"));
        assert!(rendered.contains("aida init --agent codex"));
        assert!(rendered.contains("    "));
        assert!(!rendered.contains("```"));
    }

    // trace:FR-22 | ai:antigravity
    #[test]
    fn render_md_formats_lists_with_hanging_indent() {
        let md = "- First item that is fairly long and will wrap onto the next line to demonstrate hanging indentation in list rendering.\n- Second item";
        let rendered = render_md_for_terminal_with_width(md, 50);
        assert!(rendered.contains("  • First item"));
        assert!(rendered.contains("  • Second item"));
        for line in rendered.lines() {
            assert!(visible_len(line) <= 50);
        }
    }

    // trace:FR-22 | ai:antigravity
    #[test]
    fn render_md_styles_inline_code_and_bold() {
        let md = "Check `.aida-store/` and **important** note.";
        let rendered = render_md_for_terminal_with_width(md, 78);
        assert!(!rendered.contains("`"));
        assert!(!rendered.contains("**"));
        assert!(rendered.contains(".aida-store/"));
        assert!(rendered.contains("important"));
    }
}
