//! The first-contact onboarding slice (EPIC-5).
//!
//! A thin, guided track — separate from the 36-exercise registry — that
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
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::verify;

/// Marker file proving `workspace/` holds the onboarding scratch project
/// rather than the exercise-track playground. trace:STORY-46 | ai:claude
const SCRATCH_MARKER: &str = "greet.py";
const AGENT_CHOICE_FILE: &str = ".aida-tutor-onboard.toml";

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
    met: fn(&Path, &AgentSelection) -> bool,
    /// One-line nudge shown in the footer while the checkpoint is pending.
    nudge: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GuidanceMode<'a> {
    TopLevel,
    Shell { cwd: &'a Path },
}

/// Primary implementer selected for the onboarding tour. The tour teaches
/// one round trip, so the first slice records one primary agent even when
/// several are installed. trace:STORY-49,STORY-50 | ai:codex
// trace:STORY-50 | ai:codex
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AgentChoice {
    Codex,
    Claude,
    Antigravity,
    Human,
}

impl AgentChoice {
    fn all() -> [AgentChoice; 4] {
        [
            AgentChoice::Codex,
            AgentChoice::Claude,
            AgentChoice::Antigravity,
            AgentChoice::Human,
        ]
    }

    fn key(self) -> &'static str {
        match self {
            AgentChoice::Codex => "codex",
            AgentChoice::Claude => "claude",
            AgentChoice::Antigravity => "antigravity",
            AgentChoice::Human => "human",
        }
    }

    fn display(self) -> &'static str {
        match self {
            AgentChoice::Codex => "Codex",
            AgentChoice::Claude => "Claude Code",
            AgentChoice::Antigravity => "Antigravity",
            AgentChoice::Human => "Manual / human",
        }
    }

    fn trace_tool(self) -> &'static str {
        match self {
            AgentChoice::Codex => "codex",
            AgentChoice::Claude => "claude",
            AgentChoice::Antigravity => "antigravity",
            AgentChoice::Human => "human",
        }
    }

    fn command_names(self) -> &'static [&'static str] {
        match self {
            AgentChoice::Codex => &["codex"],
            AgentChoice::Claude => &["claude"],
            AgentChoice::Antigravity => &["ag", "antigravity"],
            AgentChoice::Human => &[],
        }
    }

    fn prompt_intro(self) -> &'static str {
        match self {
            AgentChoice::Codex => "Have Codex implement it:",
            AgentChoice::Claude => "In Claude Code, ask:",
            AgentChoice::Antigravity => "In Antigravity, ask:",
            AgentChoice::Human => "Make the change yourself:",
        }
    }

    fn commit_prefix(self) -> &'static str {
        match self {
            AgentChoice::Human => "feat",
            _ => "[AI:{{trace_tool}}] feat",
        }
    }

    fn from_key(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "codex" => Some(AgentChoice::Codex),
            "claude" => Some(AgentChoice::Claude),
            "ag" | "antigravity" => Some(AgentChoice::Antigravity),
            "human" | "manual" => Some(AgentChoice::Human),
            _ => None,
        }
    }

    fn is_available(self) -> bool {
        self == AgentChoice::Human
            || self
                .command_names()
                .iter()
                .any(|name| command_available(name))
    }

    // trace:STORY-54 | ai:codex
    fn implementation_action(self, workspace: &Path, mode: GuidanceMode<'_>) -> String {
        let followup = match mode {
            GuidanceMode::TopLevel => "cd ..\n    cargo run -- next",
            GuidanceMode::Shell { .. } => "",
        };
        let prefix = shell_command_prefix(workspace, mode);
        let suffix = command_suffix(followup);
        match self {
            AgentChoice::Codex => format!(
                "Run:\n\n    {}aida spec dryrun FR-1\n    codex exec --cd {} --sandbox workspace-write {}{}",
                prefix,
                workspace.display(),
                shell_single_quote(
                    "Use AIDA first: run `aida show FR-1 --format human`. Then implement FR-1 in greet.py: add a --upper flag that uppercases the greeting. Leave a `# trace:FR-1 | ai:codex` comment next to the code that implements it. Verify with `python3 greet.py --upper World`. Do not commit."
                ),
                suffix
            ),
            AgentChoice::Claude => format!(
                "Run:\n\n    {}aida spec dryrun FR-1\n    claude -p {}{}",
                prefix,
                shell_single_quote(
                    "Use AIDA first: run `aida show FR-1 --format human`. Then implement FR-1 in greet.py: add a --upper flag that uppercases the greeting. Leave a `# trace:FR-1 | ai:claude` comment next to the code that implements it. Verify with `python3 greet.py --upper World`. Do not commit."
                ),
                suffix
            ),
            AgentChoice::Antigravity => {
                format!(
                    "Run:\n\n    {}aida spec dryrun FR-1\n\nThen ask Antigravity to implement FR-1 in greet.py with the required trace comment. The tutor will verify the resulting file.",
                    prefix
                )
            }
            AgentChoice::Human => {
                format!(
                    "Run:\n\n    {}aida spec dryrun FR-1\n\nThen edit greet.py yourself and add `# trace:FR-1 | ai:human` next to the --upper implementation.",
                    prefix
                )
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AgentSelection {
    primary: AgentChoice,
    scaffold: Vec<AgentChoice>,
}

impl AgentSelection {
    fn new(primary: AgentChoice, mut scaffold: Vec<AgentChoice>) -> Self {
        scaffold.retain(|agent| *agent != AgentChoice::Human);
        scaffold.sort_by_key(|agent| match agent {
            AgentChoice::Codex => 0,
            AgentChoice::Claude => 1,
            AgentChoice::Antigravity => 2,
            AgentChoice::Human => 3,
        });
        scaffold.dedup();
        Self { primary, scaffold }
    }

    fn default_noninteractive() -> Self {
        let primary = AgentChoice::all()
            .into_iter()
            .find(|agent| agent.is_available())
            .unwrap_or(AgentChoice::Human);
        let scaffold = if primary == AgentChoice::Human {
            Vec::new()
        } else {
            vec![primary]
        };
        Self::new(primary, scaffold)
    }

    fn scaffold_display(&self) -> String {
        if self.scaffold.is_empty() {
            return "none".to_string();
        }
        self.scaffold
            .iter()
            .map(|agent| agent.display())
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn init_command(&self) -> &'static str {
        let has_codex = self.scaffold.contains(&AgentChoice::Codex);
        let has_claude = self.scaffold.contains(&AgentChoice::Claude);
        let has_antigravity = self.scaffold.contains(&AgentChoice::Antigravity);
        match (has_codex, has_claude, has_antigravity) {
            (true, false, false) => "aida init --agent codex",
            (false, true, false) => "aida init --agent claude",
            (true, true, false) => "aida init --agent both",
            (false, false, false) => "aida init --no-skills",
            _ => "aida init --no-skills",
        }
    }

    fn init_note(&self) -> &'static str {
        let has_antigravity = self.scaffold.contains(&AgentChoice::Antigravity);
        let has_codex = self.scaffold.contains(&AgentChoice::Codex);
        let has_claude = self.scaffold.contains(&AgentChoice::Claude);
        if has_antigravity {
            "AIDA 0.14's --agent flag only supports claude, codex, or both. Because this selection includes Antigravity, the tour uses --no-skills to avoid scaffolding a prohibited agent; wire Antigravity separately if your environment requires it."
        } else if has_codex && !has_claude {
            "This uses AIDA's Codex init profile and does not scaffold Claude Code. Current AIDA 0.14 may still write .antigravity/ even though Antigravity was not selected; strict all-agent allow-list behavior is tracked upstream as BUG-794. If .antigravity/ appears, remove it before continuing this compliance-safe tour."
        } else if has_claude && !has_codex {
            "This scaffolds Claude Code-facing files only."
        } else if has_claude && has_codex {
            "This scaffolds both Claude Code and Codex because both were selected."
        } else {
            "This skips generated agent skills and commands; use it when you are doing the tour manually or no listed agent is allowed."
        }
    }
}

/// The five checkpoints, in order. `passed` (below) counts how many verify
/// true from the front — that count is the learner's position in the
/// slice. trace:STORY-33 | ai:claude
fn checkpoints(selection: &AgentSelection) -> [Checkpoint; 5] {
    [
        Checkpoint {
            label: "project initialized (aida init)",
            met: cp_project_initialized,
            nudge: format!(
                "Run `{}` inside workspace/ to give the project a store. If this checkpoint still fails after init, remove unselected agent scaffold directories such as `.claude/`, `.codex/`, or `.antigravity/`.",
                selection.init_command()
            ),
        },
        Checkpoint {
            label: "intent captured as a spec",
            met: cp_spec_captured,
            nudge: "Run:\n\n    cd workspace\n    aida add --type functional --status approved --title \"greet --upper flag shouts the greeting\" --description \"Add a --upper flag to greet.py that prints the greeting in uppercase when provided. Acceptance: python3 greet.py --upper World prints HELLO, WORLD! and python3 greet.py World still prints Hello, World!\""
                .to_string(),
        },
        Checkpoint {
            label: "trace comment links code to spec",
            met: cp_trace_present,
            // trace:STORY-53 | ai:codex
            nudge: "Run the selected AI implementer against workspace/greet.py.".to_string(),
        },
        Checkpoint {
            label: "commit references the spec",
            met: cp_commit_links,
            nudge: "Commit with the spec id in parens, e.g. `... (FR-1)`.".to_string(),
        },
        Checkpoint {
            label: "feature closed out (status flipped)",
            met: cp_status_flipped,
            nudge: "Close the loop: `aida edit FR-1 --status completed`.".to_string(),
        },
    ]
}

fn cp_project_initialized(workspace: &Path, selection: &AgentSelection) -> bool {
    if !verify::is_aida_initialized(workspace) {
        return false;
    }
    // trace:BUG-10 | ai:codex
    unselected_scaffold_dirs(workspace, selection).is_empty()
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
fn cp_spec_captured(workspace: &Path, _selection: &AgentSelection) -> bool {
    fr_spec_id(workspace).is_some()
}

/// Checkpoint 3 — the scratch code carries a `trace:<FR>` comment pointing
/// at the captured spec. trace:STORY-33 | ai:claude
fn cp_trace_present(workspace: &Path, selection: &AgentSelection) -> bool {
    match fr_spec_id(workspace) {
        Some(id) => {
            if trace_comment_with_agent(workspace, &id, selection.primary.trace_tool()) {
                return true;
            }
            // Backward compatibility: older completed tours only verified
            // the spec id, before the selected implementer was persisted.
            !selected_agent_exists(workspace)
                && verify::trace_comments_in_workspace(workspace)
                    .iter()
                    .any(|t| *t == id)
        }
        None => false,
    }
}

/// Checkpoint 4 — a commit subject references the captured spec.
/// trace:STORY-33 | ai:claude
fn cp_commit_links(workspace: &Path, _selection: &AgentSelection) -> bool {
    match fr_spec_id(workspace) {
        Some(id) => verify::commit_subject_references(workspace, &id),
        None => false,
    }
}

/// Checkpoint 5 — the captured spec has left `approved`: an `FR-*` is now
/// `completed` (or `done`). trace:STORY-33 | ai:claude
fn cp_status_flipped(workspace: &Path, _selection: &AgentSelection) -> bool {
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
/// trace:STORY-46 | ai:claude
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

    let selection = select_or_load_agents(workspace)?;
    let cps = checkpoints(&selection);
    // `passed` stops at the first unmet checkpoint, so the slice is walked
    // in order even if the learner does steps out of sequence.
    let passed = passed_count(workspace, &cps, &selection);

    print_header(&cps, passed, &selection);

    let screen = passed.min(SCREENS.len() - 1);
    for (i, slug) in SCREENS[screen].iter().enumerate() {
        if i > 0 {
            println!();
        }
        print!("{}", render_content(repo_root, workspace, slug, &selection));
    }

    print_footer(workspace, &cps, passed, &selection);
    Ok(())
}

/// Print the concise command-loop view for the onboarding slice.
/// trace:STORY-52 | ai:codex
pub fn next(workspace: &Path, repo_root: &Path) -> Result<()> {
    print_next_state(workspace, repo_root, GuidanceMode::TopLevel).map(|_| ())
}

fn print_next_state(workspace: &Path, repo_root: &Path, mode: GuidanceMode) -> Result<usize> {
    ensure_seeded(workspace, repo_root)?;
    let selection = select_or_load_agents(workspace)?;
    let cps = checkpoints(&selection);
    let passed = passed_count(workspace, &cps, &selection);

    println!("{}", "AIDA onboarding next".cyan().bold());
    println!();
    println!(
        "  implementer: {}",
        selection.primary.display().cyan().bold()
    );
    println!(
        "  init profile: {}",
        selection.scaffold_display().cyan().bold()
    );
    println!();

    if passed > 0 {
        println!(
            "{} Step {} complete: {}",
            "✓".green().bold(),
            passed,
            cps[passed - 1].label
        );
    }

    if passed >= cps.len() {
        println!(
            "{} The onboarding round trip is complete.",
            "✓".green().bold()
        );
        println!(
            "  {}",
            "Browse the full 36-exercise track with `cargo run -- list`.".dimmed()
        );
        return Ok(passed);
    }

    let cp = &cps[passed];
    println!(
        "{} Step {}: {}",
        "→".cyan().bold(),
        passed + 1,
        cp.label.bold()
    );
    println!();
    println!("{}", "Next action:".bold());
    let nudge = checkpoint_nudge(workspace, &cps, passed, &selection, mode);
    for line in nudge.lines() {
        println!("  {}", line);
    }
    if !nudge.contains("cargo run -- next") {
        println!();
        println!("{}", next_after_action_text(mode).dimmed());
    }
    Ok(passed)
}

/// Start a small tutorial shell that executes commands and re-checks the
/// onboarding checkpoints after each command. trace:STORY-55 | ai:codex
pub fn shell(workspace: &Path, repo_root: &Path) -> Result<()> {
    ensure_seeded(workspace, repo_root)?;
    let mut cwd = workspace.to_path_buf();
    let mut last_passed =
        print_next_state(workspace, repo_root, GuidanceMode::Shell { cwd: &cwd })?;

    println!();
    println!(
        "{}",
        "Interactive shell commands: next, lesson, where, reset, help, exit. Other input runs in your shell.".dimmed()
    );
    println!();

    let stdin = io::stdin();
    loop {
        print!("{} ", shell_prompt(repo_root, &cwd).cyan().bold());
        io::stdout().flush()?;

        let mut line = String::new();
        if stdin.read_line(&mut line)? == 0 {
            println!();
            break;
        }
        let input = line.trim();
        if input.is_empty() {
            continue;
        }

        match input {
            "exit" | "quit" => break,
            "help" => {
                print_shell_help();
                continue;
            }
            "next" => {
                last_passed =
                    print_next_state(workspace, repo_root, GuidanceMode::Shell { cwd: &cwd })?;
                continue;
            }
            "lesson" => {
                run(workspace, repo_root, false)?;
                last_passed = current_passed(workspace)?;
                continue;
            }
            "where" => {
                println!("{}", cwd.display());
                continue;
            }
            "reset" => {
                reseed(workspace, repo_root)?;
                cwd = workspace.to_path_buf();
                println!(
                    "{} workspace reset — the `greet` scratch project is fresh.",
                    "✓".green()
                );
                last_passed =
                    print_next_state(workspace, repo_root, GuidanceMode::Shell { cwd: &cwd })?;
                continue;
            }
            _ => {}
        }

        if let Some(target) = input.strip_prefix("cd").and_then(parse_cd_target) {
            match resolve_cd(&cwd, target) {
                Ok(next) => cwd = next,
                Err(e) => println!("{} {e}", "cd:".red().bold()),
            }
            continue;
        }

        let status = Command::new("sh")
            .arg("-c")
            .arg(input)
            .current_dir(&cwd)
            .status()
            .with_context(|| format!("running command from {}", cwd.display()))?;

        if !status.success() {
            println!(
                "{} command exited with status {}",
                "✗".red().bold(),
                status.code().unwrap_or(1)
            );
        }

        let passed = current_passed(workspace)?;
        if passed > last_passed {
            announce_progress(workspace, repo_root, last_passed, passed, &cwd)?;
        } else if status.success() {
            announce_current_action(workspace, passed, &cwd)?;
        }
        last_passed = passed;
    }

    Ok(())
}

fn current_passed(workspace: &Path) -> Result<usize> {
    let selection = select_or_load_agents(workspace)?;
    let cps = checkpoints(&selection);
    Ok(passed_count(workspace, &cps, &selection))
}

fn announce_progress(
    workspace: &Path,
    repo_root: &Path,
    previous: usize,
    passed: usize,
    cwd: &Path,
) -> Result<()> {
    let selection = select_or_load_agents(workspace)?;
    let cps = checkpoints(&selection);
    for idx in previous..passed {
        println!(
            "{} Step {} complete: {}",
            "✓".green().bold(),
            idx + 1,
            cps[idx].label
        );
    }
    if passed < cps.len() {
        println!(
            "{} Step {}: {}",
            "→".cyan().bold(),
            passed + 1,
            cps[passed].label.bold()
        );
        println!();
        println!("{}", "Next action:".bold());
        for line in checkpoint_nudge(
            workspace,
            &cps,
            passed,
            &selection,
            GuidanceMode::Shell { cwd },
        )
        .lines()
        {
            println!("  {}", line);
        }
    } else {
        print_next_state(workspace, repo_root, GuidanceMode::Shell { cwd })?;
    }
    Ok(())
}

fn next_after_action_text(mode: GuidanceMode) -> &'static str {
    match mode {
        GuidanceMode::TopLevel => "Run `cargo run -- next` again after completing that action.",
        GuidanceMode::Shell { .. } => "Type `next` again after completing that action.",
    }
}

fn announce_current_action(workspace: &Path, passed: usize, cwd: &Path) -> Result<()> {
    let selection = select_or_load_agents(workspace)?;
    let cps = checkpoints(&selection);
    if passed >= cps.len() {
        return Ok(());
    }
    println!(
        "{} Still on step {}: {}",
        "→".cyan().bold(),
        passed + 1,
        cps[passed].label.bold()
    );
    println!();
    println!("{}", "Next action:".bold());
    for line in checkpoint_nudge(
        workspace,
        &cps,
        passed,
        &selection,
        GuidanceMode::Shell { cwd },
    )
    .lines()
    {
        println!("  {}", line);
    }
    Ok(())
}

fn shell_prompt(repo_root: &Path, cwd: &Path) -> String {
    let rel = cwd.strip_prefix(repo_root).unwrap_or(cwd);
    let shown = if rel.as_os_str().is_empty() {
        ".".to_string()
    } else {
        rel.display().to_string()
    };
    format!("aida-tutor:{shown}>")
}

fn print_shell_help() {
    println!("{}", "Built-ins:".bold());
    println!("  next    show the current onboarding step and next action");
    println!("  lesson  render the full current onboarding lesson");
    println!("  where   print the shell's current directory");
    println!("  reset   reset workspace/ and restart onboarding");
    println!("  cd DIR  change the shell's current directory");
    println!("  exit    leave the tutorial shell");
    println!();
    println!("Other input is executed by `sh -c` from the current directory.");
}

fn parse_cd_target(rest: &str) -> Option<&str> {
    let trimmed = rest.trim();
    if trimmed.is_empty() {
        Some("~")
    } else if rest.starts_with(char::is_whitespace) {
        Some(trimmed)
    } else {
        None
    }
}

fn resolve_cd(cwd: &Path, target: &str) -> Result<PathBuf> {
    let expanded = if target == "~" {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .context("HOME is not set")?
    } else if let Some(rest) = target.strip_prefix("~/") {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .context("HOME is not set")?
            .join(rest)
    } else {
        PathBuf::from(target)
    };
    let path = if expanded.is_absolute() {
        expanded
    } else {
        cwd.join(expanded)
    };
    if !path.is_dir() {
        bail!("{} is not a directory", path.display());
    }
    path.canonicalize()
        .with_context(|| format!("resolving {}", path.display()))
}

fn passed_count(workspace: &Path, cps: &[Checkpoint; 5], selection: &AgentSelection) -> usize {
    cps.iter()
        .take_while(|c| (c.met)(workspace, selection))
        .count()
}

/// Render the progress panel: every checkpoint as ✓ (done) / → (current) /
/// ○ (upcoming). trace:STORY-34 | ai:claude
fn print_header(cps: &[Checkpoint; 5], passed: usize, selection: &AgentSelection) {
    println!(
        "{}",
        "AIDA onboarding — the 15-minute shared-memory tour"
            .cyan()
            .bold()
    );
    println!();
    println!(
        "  implementer: {}",
        selection.primary.display().cyan().bold()
    );
    println!(
        "  init profile: {}",
        selection.scaffold_display().cyan().bold()
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
fn print_footer(
    workspace: &Path,
    cps: &[Checkpoint; 5],
    passed: usize,
    selection: &AgentSelection,
) {
    println!();
    println!("{}", "─".repeat(64).dimmed());
    if passed >= cps.len() {
        println!(
            "{} The round trip is yours — all five checkpoints met.",
            "🎉".green()
        );
        println!(
            "  {}",
            "Browse the full 36-exercise track any time with `aida-tutor list`.".dimmed()
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
    println!("  {}", "Next action:".bold());
    for line in checkpoint_nudge(workspace, cps, passed, selection, GuidanceMode::TopLevel).lines()
    {
        println!("  {}", line.dimmed());
    }
    println!();
    println!("  workspace: {}", workspace.display().to_string().cyan());
    println!(
        "  {}",
        "Use `cargo run -- onboard` for the full lesson or `cargo run -- next` for terse guidance."
            .dimmed()
    );
}

// trace:STORY-51 | ai:codex
fn checkpoint_nudge(
    workspace: &Path,
    cps: &[Checkpoint; 5],
    passed: usize,
    selection: &AgentSelection,
    mode: GuidanceMode,
) -> String {
    if passed == 0 {
        if !verify::is_aida_initialized(workspace) {
            let followup = next_followup(mode);
            let prefix = shell_command_prefix(workspace, mode);
            let suffix = command_suffix(followup);
            return format!(
                "Run:\n\n    {}{}{}",
                prefix,
                selection.init_command(),
                suffix
            );
        }

        let blocked = unselected_scaffold_dirs(workspace, selection);
        if !blocked.is_empty() {
            let prefix = shell_command_prefix(workspace, mode);
            let suffix = command_suffix(next_followup(mode));
            let cleanup = blocked
                .iter()
                .map(|dir| format!("find {dir} -depth -mindepth 1 -delete && rmdir {dir}"))
                .collect::<Vec<_>>()
                .join("\n    ");
            return format!(
                "AIDA is initialized, but unselected agent scaffold exists: {}.\nRemove it, then ask for the next action again:\n\n    {}{}{}",
                blocked.join(", "),
                prefix,
                cleanup,
                suffix
            );
        }
    }

    if passed == 1 {
        let prefix = shell_command_prefix(workspace, mode);
        let suffix = command_suffix(next_followup(mode));
        return format!(
            "Run:\n\n    {}aida add --type functional --status approved --title \"greet --upper flag shouts the greeting\" --description \"Add a --upper flag to greet.py that prints the greeting in uppercase when provided. Acceptance: python3 greet.py --upper World prints HELLO, WORLD! and python3 greet.py World still prints Hello, World!\"{}",
            prefix,
            suffix
        );
    }

    if passed == 2 {
        return selection.primary.implementation_action(workspace, mode);
    }

    cps[passed].nudge.clone()
}

fn next_followup(mode: GuidanceMode) -> &'static str {
    match mode {
        GuidanceMode::TopLevel => "cd ..\n    cargo run -- next",
        GuidanceMode::Shell { .. } => "",
    }
}

fn unselected_scaffold_dirs(workspace: &Path, selection: &AgentSelection) -> Vec<&'static str> {
    let mut dirs = Vec::new();
    if !selection.scaffold.contains(&AgentChoice::Claude) && workspace.join(".claude").exists() {
        dirs.push(".claude");
    }
    if !selection.scaffold.contains(&AgentChoice::Codex) && workspace.join(".codex").exists() {
        dirs.push(".codex");
    }
    if !selection.scaffold.contains(&AgentChoice::Antigravity)
        && workspace.join(".antigravity").exists()
    {
        dirs.push(".antigravity");
    }
    dirs
}

/// Read a screen's markdown and render it for the terminal. Returns a
/// visible placeholder rather than erroring if the file is missing — a
/// missing content file shouldn't abort the whole tour.
fn render_content(
    repo_root: &Path,
    workspace: &Path,
    slug: &str,
    selection: &AgentSelection,
) -> String {
    let path = repo_root.join(format!("content/onboarding/{slug}.md"));
    match std::fs::read_to_string(&path) {
        Ok(md) => crate::render_md_for_terminal(&personalize_content(&md, workspace, selection)),
        Err(_) => format!("(onboarding content missing: {})\n", path.display()),
    }
}

fn personalize_content(md: &str, workspace: &Path, selection: &AgentSelection) -> String {
    let agent = selection.primary;
    md.replace("{{agent_display}}", agent.display())
        .replace("{{agent_prompt_intro}}", agent.prompt_intro())
        .replace("{{trace_tool}}", agent.trace_tool())
        .replace(
            "{{implement_action}}",
            &agent.implementation_action(workspace, GuidanceMode::TopLevel),
        )
        .replace("{{init_command}}", selection.init_command())
        .replace("{{init_note}}", selection.init_note())
        .replace("{{scaffold_agents}}", &selection.scaffold_display())
        .replace(
            "{{commit_prefix}}",
            &agent
                .commit_prefix()
                .replace("{{trace_tool}}", agent.trace_tool()),
        )
}

fn select_or_load_agents(workspace: &Path) -> Result<AgentSelection> {
    if let Some(selection) = load_agent_selection(workspace) {
        return Ok(selection);
    }
    if let Some(choice) = infer_agent_choice_from_trace(workspace) {
        let selection = AgentSelection::new(choice, vec![choice]);
        save_agent_selection(workspace, &selection)?;
        return Ok(selection);
    }

    print_agent_detection();

    let selection = if io::stdin().is_terminal() {
        prompt_agent_selection()?
    } else {
        AgentSelection::default_noninteractive()
    };
    save_agent_selection(workspace, &selection)?;
    println!(
        "{} onboarding implementer: {}",
        "✓".green(),
        selection.primary.display().cyan().bold()
    );
    println!(
        "{} scaffold targets: {}",
        "✓".green(),
        selection.scaffold_display().cyan().bold()
    );
    println!();
    Ok(selection)
}

fn prompt_agent_selection() -> Result<AgentSelection> {
    loop {
        print!("Select allowed scaffold agents [comma-separated, default 1]: ");
        io::stdout().flush()?;
        let mut answer = String::new();
        io::stdin().read_line(&mut answer)?;
        let selected = parse_agent_list(answer.trim());
        if let Some(selected) = selected {
            let primary = prompt_primary_agent(&selected)?;
            return Ok(AgentSelection::new(primary, selected));
        }
        println!("Please enter numbers or names, e.g. `1`, `1,3`, `codex`, or `human`.");
    }
}

fn prompt_primary_agent(scaffold: &[AgentChoice]) -> Result<AgentChoice> {
    let mut options = scaffold.to_vec();
    if !options.contains(&AgentChoice::Human) {
        options.push(AgentChoice::Human);
    }
    if options.len() == 1 {
        return Ok(options[0]);
    }
    println!();
    println!("Choose the primary implementer for Step 3:");
    for (idx, agent) in options.iter().enumerate() {
        println!("  {}. {}", idx + 1, agent.display());
    }
    loop {
        print!("Primary implementer [default 1]: ");
        io::stdout().flush()?;
        let mut answer = String::new();
        io::stdin().read_line(&mut answer)?;
        let trimmed = answer.trim();
        if trimmed.is_empty() {
            return Ok(options[0]);
        }
        if let Ok(n) = trimmed.parse::<usize>() {
            if (1..=options.len()).contains(&n) {
                return Ok(options[n - 1]);
            }
        }
        if let Some(choice) = AgentChoice::from_key(trimmed) {
            if options.contains(&choice) {
                return Ok(choice);
            }
        }
        println!("Please choose one of the listed implementers.");
    }
}

fn parse_agent_list(input: &str) -> Option<Vec<AgentChoice>> {
    let raw = if input.trim().is_empty() { "1" } else { input };
    let mut out = Vec::new();
    for part in raw.split(',') {
        let token = part.trim();
        let choice = match token {
            "1" => Some(AgentChoice::Codex),
            "2" => Some(AgentChoice::Claude),
            "3" => Some(AgentChoice::Antigravity),
            "4" => Some(AgentChoice::Human),
            other => AgentChoice::from_key(other),
        }?;
        if choice != AgentChoice::Human && !choice.is_available() {
            println!(
                "{} is not available on PATH; choose an installed or allowed option.",
                choice.display()
            );
            return None;
        }
        if !out.contains(&choice) {
            out.push(choice);
        }
    }
    Some(out)
}

fn print_agent_detection() {
    println!("{}", "Detected coding agents".cyan().bold());
    for (idx, agent) in AgentChoice::all().iter().enumerate() {
        let status = if agent.is_available() {
            "✓".green().bold()
        } else {
            "-".dimmed()
        };
        let suffix = if *agent == AgentChoice::Human {
            "always available".dimmed().to_string()
        } else if agent.is_available() {
            "found on PATH".dimmed().to_string()
        } else {
            "not found on PATH".dimmed().to_string()
        };
        println!(
            "  {} {}. {:<16} {}",
            status,
            idx + 1,
            agent.display(),
            suffix
        );
    }
    println!();
}

fn load_agent_selection(workspace: &Path) -> Option<AgentSelection> {
    let content = std::fs::read_to_string(workspace.join(AGENT_CHOICE_FILE)).ok()?;
    let primary = content.lines().find_map(|line| {
        let value = line.trim().strip_prefix("agent = ")?;
        AgentChoice::from_key(value.trim_matches('"'))
    })?;
    let scaffold = content
        .lines()
        .find_map(|line| {
            let value = line.trim().strip_prefix("scaffold_agents = ")?;
            Some(parse_saved_agent_list(value))
        })
        .unwrap_or_else(|| {
            if primary == AgentChoice::Human {
                Vec::new()
            } else {
                vec![primary]
            }
        });
    Some(AgentSelection::new(primary, scaffold))
}

fn save_agent_selection(workspace: &Path, selection: &AgentSelection) -> Result<()> {
    let scaffold = selection
        .scaffold
        .iter()
        .map(|agent| agent.key())
        .collect::<Vec<_>>()
        .join(",");
    std::fs::write(
        workspace.join(AGENT_CHOICE_FILE),
        format!(
            "agent = \"{}\"\nscaffold_agents = \"{}\"\n",
            selection.primary.key(),
            scaffold
        ),
    )
    .with_context(|| format!("writing {}", workspace.join(AGENT_CHOICE_FILE).display()))
}

fn parse_saved_agent_list(value: &str) -> Vec<AgentChoice> {
    value
        .trim_matches('"')
        .split(',')
        .filter_map(AgentChoice::from_key)
        .filter(|agent| *agent != AgentChoice::Human)
        .collect()
}

fn selected_agent_exists(workspace: &Path) -> bool {
    workspace.join(AGENT_CHOICE_FILE).exists()
}

fn infer_agent_choice_from_trace(workspace: &Path) -> Option<AgentChoice> {
    let spec = fr_spec_id(workspace)?;
    AgentChoice::all()
        .into_iter()
        .filter(|agent| *agent != AgentChoice::Human)
        .find(|agent| trace_comment_with_agent(workspace, &spec, agent.trace_tool()))
}

fn command_available(name: &str) -> bool {
    std::process::Command::new("sh")
        .arg("-c")
        .arg("command -v \"$1\" >/dev/null 2>&1")
        .arg("sh")
        .arg(name)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

// trace:BUG-11 | ai:codex
fn shell_command_prefix(workspace: &Path, mode: GuidanceMode<'_>) -> String {
    match mode {
        GuidanceMode::TopLevel => format!("cd {}\n    ", workspace.display()),
        GuidanceMode::Shell { cwd } if same_dir(cwd, workspace) => String::new(),
        GuidanceMode::Shell { .. } => format!("cd {}\n    ", workspace.display()),
    }
}

fn command_suffix(followup: &str) -> String {
    if followup.is_empty() {
        String::new()
    } else {
        format!("\n    {followup}")
    }
}

fn same_dir(a: &Path, b: &Path) -> bool {
    let Ok(a) = a.canonicalize() else {
        return false;
    };
    let Ok(b) = b.canonicalize() else {
        return false;
    };
    a == b
}

fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn trace_comment_with_agent(workspace: &Path, spec: &str, trace_tool: &str) -> bool {
    let needle = format!("trace:{spec}");
    let ai_needle = format!("ai:{trace_tool}");
    for entry in walkdir::WalkDir::new(workspace)
        .into_iter()
        .filter_entry(|e| {
            let n = e.file_name().to_string_lossy();
            n != ".aida-store" && n != ".git" && n != "target" && n != ".aida"
        })
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let Ok(content) = std::fs::read_to_string(entry.path()) else {
            continue;
        };
        if content
            .lines()
            .any(|line| line.contains(&needle) && line.contains(&ai_needle))
        {
            return true;
        }
    }
    false
}

/// Seed `workspace/` only if it isn't already the onboarding scratch
/// project. Refuses to clobber an in-progress exercise-track workspace —
/// the learner must opt in with `--reset`. trace:STORY-46 | ai:claude
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
/// trace:STORY-46 | ai:claude
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
/// (AC-5, `onboard_seed_is_reset_safe`). trace:STORY-46 | ai:claude
fn seed(workspace: &Path, repo_root: &Path) -> Result<()> {
    let template = repo_root.join("content/onboarding/scratch-template");
    if !template.is_dir() {
        bail!("scratch template missing at {}", template.display());
    }
    std::fs::create_dir_all(workspace)
        .with_context(|| format!("creating {}", workspace.display()))?;
    for entry in
        std::fs::read_dir(&template).with_context(|| format!("reading {}", template.display()))?
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
    git(
        workspace,
        &["config", "user.email", "learner@aida-tutor.local"],
    )?;
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

    fn selection(primary: AgentChoice, scaffold: &[AgentChoice]) -> AgentSelection {
        AgentSelection::new(primary, scaffold.to_vec())
    }

    #[test]
    fn onboard_verify_spec_created() {
        let ws = tmp();
        let sel = selection(AgentChoice::Claude, &[AgentChoice::Claude]);
        assert!(!cp_spec_captured(&ws, &sel), "no store yet → no spec");
        store_with_fr(&ws, "approved");
        assert!(
            cp_spec_captured(&ws, &sel),
            "FR-1 object present in the store"
        );
        std::fs::remove_dir_all(&ws).ok();
    }

    #[test]
    fn onboard_verify_trace_comment_present() {
        let ws = tmp();
        let sel = selection(AgentChoice::Claude, &[AgentChoice::Claude]);
        store_with_fr(&ws, "approved");
        assert!(!cp_trace_present(&ws, &sel), "no trace comment yet");
        std::fs::write(
            ws.join("greet.py"),
            "# trace:FR-1 | ai:claude\nprint('hi')\n",
        )
        .unwrap();
        assert!(cp_trace_present(&ws, &sel), "greet.py carries trace:FR-1");
        std::fs::remove_dir_all(&ws).ok();
    }

    #[test]
    fn onboard_trace_comment_honors_selected_agent() {
        let ws = tmp();
        store_with_fr(&ws, "approved");
        let codex = selection(AgentChoice::Codex, &[AgentChoice::Codex]);
        let claude = selection(AgentChoice::Claude, &[AgentChoice::Claude]);
        std::fs::write(
            ws.join("greet.py"),
            "# trace:FR-1 | ai:codex\nprint('hi')\n",
        )
        .unwrap();
        assert!(
            cp_trace_present(&ws, &codex),
            "selected Codex trace is accepted"
        );
        save_agent_selection(&ws, &claude).unwrap();
        assert!(
            !cp_trace_present(&ws, &claude),
            "wrong selected-agent trace is rejected for new tours"
        );
        std::fs::remove_dir_all(&ws).ok();
    }

    #[test]
    fn onboard_trace_comment_keeps_legacy_tours_working() {
        let ws = tmp();
        store_with_fr(&ws, "approved");
        let codex = selection(AgentChoice::Codex, &[AgentChoice::Codex]);
        std::fs::write(ws.join("greet.py"), "# trace:FR-1\nprint('hi')\n").unwrap();
        assert!(
            cp_trace_present(&ws, &codex),
            "legacy trace without a persisted agent choice still passes"
        );
        std::fs::remove_dir_all(&ws).ok();
    }

    #[test]
    fn onboard_codex_only_rejects_claude_scaffold() {
        let ws = tmp();
        std::fs::create_dir_all(ws.join(".aida")).unwrap();
        std::fs::write(ws.join(".aida/config.toml"), "").unwrap();
        std::fs::create_dir_all(ws.join(".aida-store")).unwrap();
        std::fs::write(ws.join(".aida-store/metadata.yaml"), "").unwrap();
        std::fs::create_dir_all(ws.join(".claude")).unwrap();
        let codex = selection(AgentChoice::Codex, &[AgentChoice::Codex]);
        assert!(
            !cp_project_initialized(&ws, &codex),
            "Codex-only onboarding must not accept Claude scaffold"
        );
        std::fs::remove_dir_all(ws.join(".claude")).unwrap();
        std::fs::create_dir_all(ws.join(".codex")).unwrap();
        assert!(
            cp_project_initialized(&ws, &codex),
            "Codex-only onboarding accepts Codex scaffold"
        );
        std::fs::create_dir_all(ws.join(".antigravity")).unwrap();
        assert!(
            !cp_project_initialized(&ws, &codex),
            "Codex-only onboarding must not accept Antigravity scaffold"
        );
        std::fs::remove_dir_all(&ws).ok();
    }

    #[test]
    fn onboard_nudge_explains_unselected_scaffold_blocker() {
        let ws = tmp();
        std::fs::create_dir_all(ws.join(".aida")).unwrap();
        std::fs::write(ws.join(".aida/config.toml"), "").unwrap();
        std::fs::create_dir_all(ws.join(".aida-store")).unwrap();
        std::fs::write(ws.join(".aida-store/metadata.yaml"), "").unwrap();
        std::fs::create_dir_all(ws.join(".antigravity")).unwrap();
        let codex = selection(AgentChoice::Codex, &[AgentChoice::Codex]);
        let cps = checkpoints(&codex);

        let nudge = checkpoint_nudge(&ws, &cps, 0, &codex, GuidanceMode::TopLevel);

        assert!(nudge.contains("AIDA is initialized"));
        assert!(nudge.contains(".antigravity"));
        assert!(nudge.contains("cargo run -- next"));
        std::fs::remove_dir_all(&ws).ok();
    }

    #[test]
    fn onboard_step_three_nudge_names_agent_and_file() {
        let ws = tmp();
        let codex = selection(AgentChoice::Codex, &[AgentChoice::Codex]);
        let cps = checkpoints(&codex);

        let nudge = checkpoint_nudge(&ws, &cps, 2, &codex, GuidanceMode::TopLevel);

        assert!(nudge.contains("aida spec dryrun FR-1"));
        assert!(nudge.contains("codex exec"));
        assert!(nudge.contains(&format!("--cd {}", ws.display())));
        assert!(nudge.contains("--upper flag"));
        assert!(nudge.contains("# trace:FR-1 | ai:codex"));
        std::fs::remove_dir_all(&ws).ok();
    }

    #[test]
    fn shell_nudge_omits_redundant_cd_and_next_inside_workspace() {
        let ws = tmp();
        std::fs::create_dir_all(&ws).unwrap();
        let codex = selection(AgentChoice::Codex, &[AgentChoice::Codex]);
        let cps = checkpoints(&codex);

        let nudge = checkpoint_nudge(&ws, &cps, 1, &codex, GuidanceMode::Shell { cwd: &ws });

        assert!(nudge.contains("aida add --type functional"));
        assert!(!nudge
            .lines()
            .any(|line| line.trim() == format!("cd {}", ws.display())));
        assert!(!nudge.contains("cargo run -- next"));
        assert!(!nudge.lines().any(|line| line.trim() == "next"));
        std::fs::remove_dir_all(&ws).ok();
    }

    #[test]
    fn onboard_verify_commit_links_spec() {
        let ws = tmp();
        let sel = selection(AgentChoice::Claude, &[AgentChoice::Claude]);
        store_with_fr(&ws, "approved");
        run_git(&ws, &["init", "-q"]);
        run_git(&ws, &["config", "user.email", "t@t.local"]);
        run_git(&ws, &["config", "user.name", "T"]);
        std::fs::write(ws.join("greet.py"), "x\n").unwrap();
        run_git(&ws, &["add", "-A"]);
        run_git(
            &ws,
            &[
                "-c",
                "commit.gpgsign=false",
                "commit",
                "-q",
                "-m",
                "Initial greet CLI",
            ],
        );
        assert!(!cp_commit_links(&ws, &sel), "no commit names FR-1 yet");
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
        assert!(cp_commit_links(&ws, &sel), "commit subject references FR-1");
        std::fs::remove_dir_all(&ws).ok();
    }

    #[test]
    fn onboard_verify_status_flipped() {
        let ws = tmp();
        let sel = selection(AgentChoice::Claude, &[AgentChoice::Claude]);
        store_with_fr(&ws, "approved");
        assert!(
            !cp_status_flipped(&ws, &sel),
            "still approved → not flipped"
        );
        std::fs::remove_dir_all(&ws).ok();

        let ws = tmp();
        store_with_fr(&ws, "completed");
        assert!(cp_status_flipped(&ws, &sel), "completed → flipped");
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
        assert!(
            ws.join(SCRATCH_MARKER).exists(),
            "scratch project re-seeded"
        );

        assert_eq!(
            std::fs::read_to_string(&progress).unwrap(),
            before,
            "progress file untouched by seed/reseed"
        );
        std::fs::remove_dir_all(&root).ok();
    }
}
