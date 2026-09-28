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
use serde::Deserialize;
use std::io::{self, IsTerminal, Write};
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::exercise::VerifyResult;
use crate::progress::Progress;
use crate::verify;

/// Marker file proving `workspace/` holds the onboarding scratch project
/// rather than the exercise-track playground. trace:STORY-46 | ai:claude
const SCRATCH_MARKER: &str = "greet.py";
const AGENT_CHOICE_FILE: &str = ".aida-tutor-onboard.toml";
const SHELL_RULE_WIDTH: usize = 120;

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
        // trace:BUG-13 | ai:codex
        let readiness = implementation_readiness(workspace);
        if let ImplementationReadiness::Blocked(reason) = &readiness {
            return format!(
                "Readiness blocked for FR-1: {reason}\nResolve the blocker, then ask for the next action again."
            );
        }
        let preamble = match readiness {
            ImplementationReadiness::StandaloneParentOnly => format!(
                "Readiness note: FR-1 is standalone in this onboarding tour, so its missing parent only matters before queueing it. Continue with direct implementation; do not rerun the dryrun.\n\n    {}",
                prefix
            ),
            ImplementationReadiness::Ready | ImplementationReadiness::Unavailable => {
                format!("Run:\n\n    {}aida spec dryrun FR-1\n    ", prefix)
            }
            ImplementationReadiness::Blocked(_) => unreachable!(),
        };
        match self {
            AgentChoice::Codex => format!(
                "{}codex exec --cd {} --sandbox workspace-write {}{}",
                preamble,
                workspace.display(),
                shell_single_quote(
                    "Use AIDA first: run `aida show FR-1 --format human`. Then implement FR-1 in greet.py: add a --upper flag that uppercases the greeting. Leave a `# trace:FR-1 | ai:codex` comment next to the code that implements it. Verify with `python3 greet.py --upper World`. Do not commit."
                ),
                suffix
            ),
            AgentChoice::Claude => format!(
                "{}claude -p {}{}",
                preamble,
                shell_single_quote(
                    "Use AIDA first: run `aida show FR-1 --format human`. Then implement FR-1 in greet.py: add a --upper flag that uppercases the greeting. Leave a `# trace:FR-1 | ai:claude` comment next to the code that implements it. Verify with `python3 greet.py --upper World`. Do not commit."
                ),
                suffix
            ),
            AgentChoice::Antigravity => {
                format!(
                    "{}\n\nThen ask Antigravity to implement FR-1 in greet.py with the required trace comment. The tutor will verify the resulting file.",
                    preamble
                )
            }
            AgentChoice::Human => {
                format!(
                    "{}\n\nThen edit greet.py yourself and add `# trace:FR-1 | ai:human` next to the --upper implementation.",
                    preamble
                )
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ImplementationReadiness {
    Ready,
    StandaloneParentOnly,
    Blocked(String),
    Unavailable,
}

// trace:BUG-13 | ai:codex
fn implementation_readiness(workspace: &Path) -> ImplementationReadiness {
    let output = match Command::new("aida")
        .current_dir(workspace)
        .args(["spec", "dryrun", "FR-1", "--format", "human"])
        .output()
    {
        Ok(output) if output.status.success() => output,
        _ => return ImplementationReadiness::Unavailable,
    };

    implementation_readiness_from_output(&String::from_utf8_lossy(&output.stdout))
}

fn implementation_readiness_from_output(output: &str) -> ImplementationReadiness {
    let failures: Vec<String> = output
        .lines()
        .filter_map(|line| line.trim().strip_prefix('✗'))
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect();

    match failures.as_slice() {
        [] => ImplementationReadiness::Ready,
        [failure] if failure.starts_with("parent") => ImplementationReadiness::StandaloneParentOnly,
        _ => ImplementationReadiness::Blocked(failures.join("; ")),
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
            nudge: commit_nudge(selection),
        },
        Checkpoint {
            label: "feature closed out (status flipped)",
            met: cp_status_flipped,
            nudge: "Run:\n\n    aida edit FR-1 --status completed".to_string(),
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
    if passed >= cps.len() {
        mark_onboarding_completed(repo_root);
    }
    Ok(())
}

/// Print the concise command-loop view for the onboarding slice.
/// trace:STORY-52 | ai:codex
pub fn next(workspace: &Path, repo_root: &Path) -> Result<()> {
    print_next_state(workspace, repo_root, GuidanceMode::TopLevel).map(|_| ())
}

// trace:FR-9,FR-19 | ai:codex,antigravity
pub fn menu(target_dir: &Path, repo_root: &Path) -> Result<()> {
    run_launchpad_menu(target_dir, repo_root)
}

// trace:FR-15,FR-18,FR-19 | ai:codex,antigravity
pub fn aida_shell(workspace: &Path, repo_root: &Path) -> Result<()> {
    match run_aida_shell(workspace)? {
        ShellExitAction::Menu => run_launchpad_menu(workspace, repo_root),
        ShellExitAction::Exit => Ok(()),
    }
}


// trace:FR-19,FR-20 | ai:antigravity
/// Returns true if `dir` contains an initialized AIDA project.
pub fn is_aida_project_root(dir: &Path) -> bool {
    let store = dir.join(".aida-store");
    if store.is_dir() {
        return true;
    }
    if store.is_file() {
        if let Ok(content) = std::fs::read_to_string(&store) {
            for line in content.lines() {
                let trimmed = line.trim();
                if let Some(name) = trimmed.strip_prefix("name:") {
                    let name = name.trim().trim_matches('\'').trim_matches('"');
                    if !name.is_empty() {
                        return true;
                    }
                }
            }
        }
    }
    dir.join(".aida/config.toml").is_file() || dir.join("requirements.db").is_file()
}

// trace:FR-19 | ai:antigravity
/// Returns the nearest ancestor directory (or `start` itself) that is an
/// initialized AIDA project.
pub fn find_aida_project_root(start: &Path) -> Option<PathBuf> {
    let mut cur = Some(start.to_path_buf());
    while let Some(dir) = cur {
        if is_aida_project_root(&dir) {
            return Some(dir);
        }
        cur = dir.parent().map(|p| p.to_path_buf());
    }
    None
}

// trace:FR-19 | ai:antigravity
/// Returns true if `dir` contains a Git repository.
pub fn is_git_project_root(dir: &Path) -> bool {
    dir.join(".git/HEAD").is_file() || dir.join(".git").is_file()
}

// trace:FR-19 | ai:antigravity
/// Returns the nearest ancestor directory (or `start` itself) that contains
/// a Git repository.
pub fn find_git_project_root(start: &Path) -> Option<PathBuf> {
    let mut cur = Some(start.to_path_buf());
    while let Some(dir) = cur {
        if is_git_project_root(&dir) {
            return Some(dir);
        }
        cur = dir.parent().map(|p| p.to_path_buf());
    }
    None
}

// trace:FR-19 | ai:antigravity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LaunchpadOption {
    Init,
    Agent,
    Status,
    Shell,
    Help,
    Exit,
}

impl LaunchpadOption {
    pub(crate) fn label(&self) -> &'static str {
        match self {
            LaunchpadOption::Init => "init        — initialize this project for AIDA",
            LaunchpadOption::Agent => "agent       — create or configure an agent",
            LaunchpadOption::Status => "status      — inspect AIDA status",
            LaunchpadOption::Shell => "shell       — run AIDA commands",
            LaunchpadOption::Help => "help        — tutorials and documentation",
            LaunchpadOption::Exit => "exit        — leave",
        }
    }
}

// trace:FR-19 | ai:antigravity
pub(crate) fn launchpad_options(is_aida: bool) -> &'static [LaunchpadOption] {
    if is_aida {
        &[
            LaunchpadOption::Agent,
            LaunchpadOption::Status,
            LaunchpadOption::Shell,
            LaunchpadOption::Help,
            LaunchpadOption::Exit,
        ]
    } else {
        &[
            LaunchpadOption::Init,
            LaunchpadOption::Help,
            LaunchpadOption::Exit,
        ]
    }
}

// trace:FR-15,FR-19,FR-21 | ai:codex,antigravity
pub(crate) fn launchpad_menu_text(
    options: &[LaunchpadOption],
    selected: usize,
    clear_screen: bool,
) -> String {
    let mut out = format!("{}\n\n", "AIDA".cyan().bold());
    for (i, opt) in options.iter().enumerate() {
        let cursor = if i == selected { "❯" } else { " " };
        out.push_str(&format!("{cursor} {}\n", opt.label()));
    }
    let clear_status = if clear_screen { "on" } else { "off" };
    out.push_str(&format!(
        "  Use ↑/↓ (or j/k), Enter to select, c to toggle clear [{clear_status}], Esc to leave."
    ));
    out
}

// trace:FR-19,FR-20,FR-21 | ai:antigravity
fn draw_launchpad_menu(
    options: &[LaunchpadOption],
    selected: usize,
    redraw: bool,
    dir: &Path,
    clear_screen: bool,
) -> Result<()> {
    let text = launchpad_menu_text(options, selected, clear_screen);
    if redraw {
        if clear_screen {
            crate::banner::redraw_menu(&text, Some(dir))?;
        } else {
            let lines_up = options.len() + 3;
            print!("\x1b[{}A\r", lines_up);
            println!("{}", text);
            print!("\x1b[J");
        }
    } else {
        println!("{}", text);
    }
    let _ = io::stdout().flush();
    Ok(())
}

// trace:FR-19,FR-21 | ai:antigravity
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum MenuNavAction {
    Continue,
    Select,
    Cancel,
    ToggleClear,
}

// trace:FR-19,FR-21 | ai:antigravity
pub(crate) fn handle_launchpad_key(
    byte: u8,
    selected: &mut usize,
    max_idx: usize,
    stdin: &mut io::Stdin,
) -> Result<MenuNavAction> {
    match byte {
        b'\r' | b'\n' => Ok(MenuNavAction::Select),
        3 => Ok(MenuNavAction::Cancel),
        b'c' | b'C' => Ok(MenuNavAction::ToggleClear),
        b'k' => {
            *selected = selected.saturating_sub(1);
            Ok(MenuNavAction::Continue)
        }
        b'j' => {
            *selected = (*selected + 1).min(max_idx);
            Ok(MenuNavAction::Continue)
        }
        0x1b => match read_escape_sequence(stdin) {
            Some([b'[', b'A']) => {
                *selected = selected.saturating_sub(1);
                Ok(MenuNavAction::Continue)
            }
            Some([b'[', b'B']) => {
                *selected = (*selected + 1).min(max_idx);
                Ok(MenuNavAction::Continue)
            }
            Some([b'[', b'D']) | None => Ok(MenuNavAction::Cancel),
            _ => Ok(MenuNavAction::Continue),
        },
        _ => Ok(MenuNavAction::Continue),
    }
}

// trace:FR-19 | ai:antigravity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InitChoice {
    CurrentDir,
    Subdirectory,
    Cancel,
}

// trace:FR-19 | ai:antigravity
fn prompt_init_choice(target_dir: &Path) -> Result<InitChoice> {
    let current_label = format!("initialize current directory ({})", target_dir.display());
    let options = [
        current_label.as_str(),
        "create a new project in a subdirectory",
        "cancel",
    ];

    if !io::stdin().is_terminal() {
        println!();
        println!("{}", "AIDA / Init".cyan().bold());
        for (i, opt) in options.iter().enumerate() {
            println!("  {}. {}", i + 1, opt);
        }
        return Ok(InitChoice::Cancel);
    }

    print!("\x1b[2J\x1b[H");
    let _terminal_mode = TerminalMode::enter()?;
    let mut selected = 0usize;
    draw_init_choice_menu(&options, selected, false);
    let mut stdin = io::stdin();
    let mut byte = [0u8; 1];
    loop {
        if !read_terminal_byte(&stdin, &mut byte[0]) {
            return Ok(InitChoice::Cancel);
        }
        match byte[0] {
            b'\r' | b'\n' => break,
            3 => return Ok(InitChoice::Cancel),
            b'k' => selected = selected.saturating_sub(1),
            b'j' => selected = (selected + 1).min(options.len() - 1),
            0x1b => match read_escape_sequence(&mut stdin) {
                Some([b'[', b'A']) => selected = selected.saturating_sub(1),
                Some([b'[', b'B']) => selected = (selected + 1).min(options.len() - 1),
                Some([b'[', b'D']) | None => return Ok(InitChoice::Cancel),
                _ => {}
            },
            _ => continue,
        }
        draw_init_choice_menu(&options, selected, true);
    }
    println!();
    drop(_terminal_mode);

    match selected {
        0 => Ok(InitChoice::CurrentDir),
        1 => Ok(InitChoice::Subdirectory),
        _ => Ok(InitChoice::Cancel),
    }
}

// trace:FR-19 | ai:antigravity
fn draw_init_choice_menu(options: &[&str], selected: usize, redraw: bool) {
    if redraw {
        print!("\x1b[6A");
    }
    println!("{}", "AIDA / Init".cyan().bold());
    println!();
    for (index, option) in options.iter().enumerate() {
        println!("{} {}", if index == selected { "❯" } else { " " }, option);
    }
    println!("  Use ↑/↓ (or j/k), Enter to select, Esc to return.");
    let _ = io::stdout().flush();
}

// trace:FR-19,FR-21 | ai:antigravity
fn pause_for_enter_prompt(prompt: &str) {
    if io::stdin().is_terminal() {
        println!("\n{prompt}");
        let mut s = String::new();
        let _ = io::stdin().read_line(&mut s);
    }
}

// trace:FR-19,FR-21 | ai:antigravity
fn pause_for_enter() {
    pause_for_enter_prompt("Press Enter to continue...");
}

// trace:FR-19 | ai:antigravity
fn run_init_flow(target_dir: &mut PathBuf) -> Result<()> {
    if find_git_project_root(target_dir).is_some() {
        println!();
        println!("Initializing AIDA in {}...", target_dir.display());
        let status = Command::new("aida")
            .arg("init")
            .current_dir(&*target_dir)
            .status()
            .with_context(|| format!("running `aida init` in {}", target_dir.display()))?;
        if status.success() {
            println!();
            println!("{} Project initialized for AIDA.", "✓".green().bold());
        } else {
            println!();
            println!(
                "{} `aida init` exited with status {}",
                "✗".red().bold(),
                status.code().unwrap_or(1)
            );
        }
        pause_for_enter();
        return Ok(());
    }

    match prompt_init_choice(target_dir)? {
        InitChoice::CurrentDir => {
            println!();
            println!("Initializing git repository in {}...", target_dir.display());
            let git_status = Command::new("git")
                .arg("init")
                .current_dir(&*target_dir)
                .status()
                .with_context(|| format!("running `git init` in {}", target_dir.display()))?;
            if !git_status.success() {
                println!("{} `git init` failed", "✗".red().bold());
                pause_for_enter();
                return Ok(());
            }

            println!("Initializing AIDA in {}...", target_dir.display());
            let aida_status = Command::new("aida")
                .arg("init")
                .current_dir(&*target_dir)
                .status()
                .with_context(|| format!("running `aida init` in {}", target_dir.display()))?;
            if aida_status.success() {
                println!();
                println!("{} Project initialized for AIDA.", "✓".green().bold());
            } else {
                println!();
                println!(
                    "{} `aida init` exited with status {}",
                    "✗".red().bold(),
                    aida_status.code().unwrap_or(1)
                );
            }
            pause_for_enter();
        }
        InitChoice::Subdirectory => {
            println!();
            print!("Enter project directory name (or leave empty to cancel): ");
            io::stdout().flush()?;
            let mut name = String::new();
            io::stdin().read_line(&mut name)?;
            let name = name.trim();
            if name.is_empty() {
                println!("Aborted.");
                return Ok(());
            }

            let sub_dir = target_dir.join(name);
            if !sub_dir.exists() {
                std::fs::create_dir_all(&sub_dir)
                    .with_context(|| format!("creating directory {}", sub_dir.display()))?;
            }

            println!("Initializing git repository in {}...", sub_dir.display());
            let git_status = Command::new("git")
                .arg("init")
                .current_dir(&sub_dir)
                .status()
                .with_context(|| format!("running `git init` in {}", sub_dir.display()))?;
            if !git_status.success() {
                println!("{} `git init` failed", "✗".red().bold());
                pause_for_enter();
                return Ok(());
            }

            println!("Initializing AIDA in {}...", sub_dir.display());
            let aida_status = Command::new("aida")
                .arg("init")
                .current_dir(&sub_dir)
                .status()
                .with_context(|| format!("running `aida init` in {}", sub_dir.display()))?;
            if aida_status.success() {
                println!();
                println!("{} Project initialized in {}.", "✓".green().bold(), sub_dir.display());
                *target_dir = sub_dir;
            } else {
                println!();
                println!(
                    "{} `aida init` exited with status {}",
                    "✗".red().bold(),
                    aida_status.code().unwrap_or(1)
                );
            }
            pause_for_enter();
        }
        InitChoice::Cancel => {}
    }

    Ok(())
}

// trace:FR-15,FR-19,FR-21 | ai:codex,antigravity
fn run_launchpad_menu(start_dir: &Path, repo_root: &Path) -> Result<()> {
    let mut current_dir = start_dir.to_path_buf();
    let mut clear_screen = true;
    loop {
        let is_aida = find_aida_project_root(&current_dir).is_some();
        let options = launchpad_options(is_aida);

        if !io::stdin().is_terminal() {
            println!();
            println!("{}", "AIDA".cyan().bold());
            for opt in options {
                println!("  {}", opt.label());
            }
            return Ok(());
        }

        if clear_screen {
            print!("\x1b[2J\x1b[H");
        }
        let first_key = crate::banner::show_and_read(
            &launchpad_menu_text(options, 0, clear_screen),
            Some(&current_dir),
        )?;
        let _terminal_mode = TerminalMode::enter()?;
        let mut selected = 0usize;
        let max_idx = options.len() - 1;
        let mut stdin = io::stdin();
        let mut chosen = false;

        if let Some(first_key) = first_key {
            match handle_launchpad_key(first_key, &mut selected, max_idx, &mut stdin)? {
                MenuNavAction::Cancel => return Ok(()),
                MenuNavAction::Select => chosen = true,
                MenuNavAction::ToggleClear => {
                    clear_screen = !clear_screen;
                    draw_launchpad_menu(options, selected, true, &current_dir, clear_screen)?;
                }
                MenuNavAction::Continue => {
                    draw_launchpad_menu(options, selected, true, &current_dir, clear_screen)?;
                }
            }
        } else {
            draw_launchpad_menu(options, selected, false, &current_dir, clear_screen)?;
        }

        if !chosen {
            let mut byte = [0u8; 1];
            loop {
                if !read_terminal_byte(&stdin, &mut byte[0]) {
                    return Ok(());
                }
                match handle_launchpad_key(byte[0], &mut selected, max_idx, &mut stdin)? {
                    MenuNavAction::Cancel => return Ok(()),
                    MenuNavAction::Select => break,
                    MenuNavAction::ToggleClear => {
                        clear_screen = !clear_screen;
                        draw_launchpad_menu(options, selected, true, &current_dir, clear_screen)?;
                    }
                    MenuNavAction::Continue => {
                        draw_launchpad_menu(options, selected, true, &current_dir, clear_screen)?;
                    }
                }
            }
        }
        println!();
        drop(_terminal_mode);

        match options[selected] {
            LaunchpadOption::Init => {
                run_init_flow(&mut current_dir)?;
            }
            LaunchpadOption::Agent => {
                let status = run_aida_command(&current_dir, &["agent", "new"])?;
                if !status.success() {
                    pause_for_enter();
                } else {
                    pause_for_enter_prompt("Press Enter to return to menu...");
                }
            }
            LaunchpadOption::Status => {
                run_status_menu(&current_dir)?;
            }
            LaunchpadOption::Shell => {
                run_aida_shell(&current_dir)?;
            }
            LaunchpadOption::Help => {
                let tutorial_workspace = repo_root.join("workspace");
                run_tutorial_menu(
                    &tutorial_workspace,
                    repo_root,
                    GuidanceMode::TopLevel,
                    &mut clear_screen,
                )?;
            }
            LaunchpadOption::Exit => {
                return Ok(());
            }
        }
    }
}

// trace:FR-15,FR-21 | ai:codex,antigravity
fn run_status_menu(workspace: &Path) -> Result<()> {
    let options = ["drain status — show the current drain state", "back"];
    if !io::stdin().is_terminal() {
        println!();
        println!("{}", "AIDA / Status".cyan().bold());
        println!("  {}", options[0]);
        println!("  {}", options[1]);
        return Ok(());
    }

    print!("\x1b[2J\x1b[H");
    let _terminal_mode = TerminalMode::enter()?;
    let mut selected = 0usize;
    draw_status_menu(&options, selected, false);
    let mut stdin = io::stdin();
    let mut byte = [0u8; 1];
    loop {
        if !read_terminal_byte(&stdin, &mut byte[0]) {
            return Ok(());
        }
        match byte[0] {
            b'\r' | b'\n' => break,
            3 => return Ok(()),
            b'k' => selected = selected.saturating_sub(1),
            b'j' => selected = (selected + 1).min(options.len() - 1),
            0x1b => match read_escape_sequence(&mut stdin) {
                Some([b'[', b'A']) => selected = selected.saturating_sub(1),
                Some([b'[', b'B']) => selected = (selected + 1).min(options.len() - 1),
                Some([b'[', b'D']) | None => return Ok(()),
                _ => {}
            },
            _ => continue,
        }
        draw_status_menu(&options, selected, true);
    }
    println!();
    drop(_terminal_mode);
    if selected == 0 {
        let status = run_aida_command(workspace, &["drain", "status"])?;
        if !status.success() {
            pause_for_enter();
        } else {
            pause_for_enter_prompt("Press Enter to return to menu...");
        }
    }
    Ok(())
}

fn draw_status_menu(options: &[&str], selected: usize, redraw: bool) {
    if redraw {
        print!("\x1b[5A");
    }
    println!("{}", "AIDA / Status".cyan().bold());
    println!();
    for (index, option) in options.iter().enumerate() {
        println!("{} {}", if index == selected { "❯" } else { " " }, option);
    }
    println!("  Use ↑/↓ (or j/k), Enter to select, Esc to return.");
    let _ = io::stdout().flush();
}

// trace:FR-15,FR-21 | ai:codex,antigravity
fn run_aida_command(workspace: &Path, args: &[&str]) -> Result<std::process::ExitStatus> {
    let status = Command::new("aida")
        .args(args)
        .current_dir(workspace)
        .status()
        .with_context(|| format!("running `aida {}`", args.join(" ")))?;
    if !status.success() {
        println!(
            "{} aida command exited with status {}",
            "✗".red().bold(),
            status.code().unwrap_or(1)
        );
    }
    Ok(status)
}

// trace:FR-17 | ai:antigravity
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct ShellHistory {
    entries: Vec<String>,
    cursor: Option<usize>,
    draft: String,
}

impl ShellHistory {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn push(&mut self, entry: String) {
        if entry.is_empty() {
            return;
        }
        if self.entries.last().map(String::as_str) != Some(entry.as_str()) {
            self.entries.push(entry);
        }
        self.cursor = None;
        self.draft.clear();
    }

    pub(crate) fn navigate_up(&mut self, current_buffer: &str) -> Option<&str> {
        if self.entries.is_empty() {
            return None;
        }
        match self.cursor {
            None => {
                self.draft = current_buffer.to_owned();
                let idx = self.entries.len() - 1;
                self.cursor = Some(idx);
                Some(&self.entries[idx])
            }
            Some(idx) => {
                if idx > 0 {
                    let next_idx = idx - 1;
                    self.cursor = Some(next_idx);
                    Some(&self.entries[next_idx])
                } else {
                    None
                }
            }
        }
    }

    pub(crate) fn navigate_down(&mut self) -> Option<&str> {
        match self.cursor {
            None => None,
            Some(idx) => {
                if idx + 1 < self.entries.len() {
                    let next_idx = idx + 1;
                    self.cursor = Some(next_idx);
                    Some(&self.entries[next_idx])
                } else {
                    self.cursor = None;
                    Some(&self.draft)
                }
            }
        }
    }

    pub(crate) fn reset_navigation(&mut self) {
        self.cursor = None;
        self.draft.clear();
    }
}

// trace:FR-17 | ai:antigravity
enum TerminalKey {
    Enter,
    Backspace,
    CtrlC,
    CtrlD,
    CtrlL,
    CtrlU,
    CtrlK,
    CtrlW,
    Home,
    End,
    Up,
    Down,
    Left,
    Right,
    Delete,
    Char(char),
    Other,
}

// trace:FR-17 | ai:antigravity
fn decode_terminal_key(byte: u8, stdin: &mut io::Stdin) -> TerminalKey {
    match byte {
        b'\r' | b'\n' => TerminalKey::Enter,
        127 | 8 => TerminalKey::Backspace,
        3 => TerminalKey::CtrlC,
        4 => TerminalKey::CtrlD,
        1 => TerminalKey::Home,
        5 => TerminalKey::End,
        11 => TerminalKey::CtrlK,
        12 => TerminalKey::CtrlL,
        21 => TerminalKey::CtrlU,
        23 => TerminalKey::CtrlW,
        0x1b => match read_escape_sequence(stdin) {
            Some([b'[', b'A']) => TerminalKey::Up,
            Some([b'[', b'B']) => TerminalKey::Down,
            Some([b'[', b'C']) => TerminalKey::Right,
            Some([b'[', b'D']) => TerminalKey::Left,
            Some([b'[', b'H']) | Some([b'O', b'H']) => TerminalKey::Home,
            Some([b'[', b'F']) | Some([b'O', b'F']) => TerminalKey::End,
            Some([b'[', b'1']) => {
                consume_trailing_tilde(stdin);
                TerminalKey::Home
            }
            Some([b'[', b'4']) => {
                consume_trailing_tilde(stdin);
                TerminalKey::End
            }
            Some([b'[', b'3']) => {
                consume_trailing_tilde(stdin);
                TerminalKey::Delete
            }
            _ => TerminalKey::Other,
        },
        b if b >= 32 => {
            if let Some(ch) = read_utf8_char(b, stdin) {
                TerminalKey::Char(ch)
            } else {
                TerminalKey::Other
            }
        }
        _ => TerminalKey::Other,
    }
}

// trace:FR-17 | ai:antigravity
fn consume_trailing_tilde(stdin: &mut io::Stdin) {
    let fd = stdin.as_raw_fd();
    let mut pollfd = libc::pollfd {
        fd,
        events: libc::POLLIN,
        revents: 0,
    };
    let ready = unsafe { libc::poll(&mut pollfd, 1, 25) };
    if ready > 0 && pollfd.revents & libc::POLLIN != 0 {
        let mut b = 0u8;
        let _ = read_terminal_byte(stdin, &mut b);
    }
}

// trace:FR-17 | ai:antigravity
fn read_utf8_char(first: u8, stdin: &io::Stdin) -> Option<char> {
    if first < 0x80 {
        return Some(first as char);
    }
    let needed = if first >= 0xF0 {
        4
    } else if first >= 0xE0 {
        3
    } else if first >= 0xC0 {
        2
    } else {
        return None;
    };
    let mut bytes = vec![first];
    for _ in 1..needed {
        let mut b = 0u8;
        if !read_terminal_byte(stdin, &mut b) {
            return None;
        }
        bytes.push(b);
    }
    std::str::from_utf8(&bytes).ok().and_then(|s| s.chars().next())
}

// trace:FR-17 | ai:antigravity
fn redraw_shell_line(prompt: &str, buffer: &str, cursor_pos: usize) -> Result<()> {
    let mut stdout = io::stdout();
    write!(stdout, "\r{prompt}{buffer}\x1b[K")?;
    let total_chars = buffer.chars().count();
    let cursor_chars = buffer[..cursor_pos].chars().count();
    let back = total_chars.saturating_sub(cursor_chars);
    if back > 0 {
        write!(stdout, "\x1b[{}D", back)?;
    }
    stdout.flush()?;
    Ok(())
}

// trace:FR-17 | ai:antigravity
fn read_aida_shell_line(prompt: &str, history: &mut ShellHistory) -> Result<Option<String>> {
    let mut stdin = io::stdin();
    if !stdin.is_terminal() {
        print!("{prompt}");
        io::stdout().flush()?;
        let mut line = String::new();
        if stdin.read_line(&mut line)? == 0 {
            return Ok(None);
        }
        while line.ends_with('\n') || line.ends_with('\r') {
            line.pop();
        }
        return Ok(Some(line));
    }

    let _terminal_mode = TerminalMode::enter()?;
    let mut buffer = String::new();
    let mut cursor_pos = 0usize;
    history.reset_navigation();
    print!("{prompt}");
    io::stdout().flush()?;

    let mut byte = [0u8; 1];
    loop {
        if !read_terminal_byte(&stdin, &mut byte[0]) {
            println!();
            return Ok(None);
        }
        match decode_terminal_key(byte[0], &mut stdin) {
            TerminalKey::Enter => {
                println!();
                return Ok(Some(buffer));
            }
            TerminalKey::CtrlC => {
                println!("^C");
                buffer.clear();
                cursor_pos = 0;
                history.reset_navigation();
                print!("{prompt}");
                io::stdout().flush()?;
            }
            TerminalKey::CtrlD => {
                if buffer.is_empty() {
                    println!();
                    return Ok(None);
                } else if cursor_pos < buffer.len() {
                    buffer.remove(cursor_pos);
                    redraw_shell_line(prompt, &buffer, cursor_pos)?;
                }
            }
            TerminalKey::Up => {
                if let Some(cmd) = history.navigate_up(&buffer) {
                    buffer = cmd.to_owned();
                    cursor_pos = buffer.len();
                    redraw_shell_line(prompt, &buffer, cursor_pos)?;
                }
            }
            TerminalKey::Down => {
                if let Some(cmd) = history.navigate_down() {
                    buffer = cmd.to_owned();
                    cursor_pos = buffer.len();
                    redraw_shell_line(prompt, &buffer, cursor_pos)?;
                }
            }
            TerminalKey::Left => {
                if cursor_pos > 0 {
                    if let Some((prev_pos, _)) = buffer[..cursor_pos].char_indices().last() {
                        cursor_pos = prev_pos;
                        redraw_shell_line(prompt, &buffer, cursor_pos)?;
                    }
                }
            }
            TerminalKey::Right => {
                if cursor_pos < buffer.len() {
                    if let Some(ch) = buffer[cursor_pos..].chars().next() {
                        cursor_pos += ch.len_utf8();
                        redraw_shell_line(prompt, &buffer, cursor_pos)?;
                    }
                }
            }
            TerminalKey::Home => {
                if cursor_pos > 0 {
                    cursor_pos = 0;
                    redraw_shell_line(prompt, &buffer, cursor_pos)?;
                }
            }
            TerminalKey::End => {
                if cursor_pos < buffer.len() {
                    cursor_pos = buffer.len();
                    redraw_shell_line(prompt, &buffer, cursor_pos)?;
                }
            }
            TerminalKey::Backspace => {
                if cursor_pos > 0 {
                    let prev_pos = buffer[..cursor_pos]
                        .char_indices()
                        .last()
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    buffer.remove(prev_pos);
                    cursor_pos = prev_pos;
                    redraw_shell_line(prompt, &buffer, cursor_pos)?;
                }
            }
            TerminalKey::Delete => {
                if cursor_pos < buffer.len() {
                    buffer.remove(cursor_pos);
                    redraw_shell_line(prompt, &buffer, cursor_pos)?;
                }
            }
            TerminalKey::CtrlU => {
                buffer.drain(..cursor_pos);
                cursor_pos = 0;
                redraw_shell_line(prompt, &buffer, cursor_pos)?;
            }
            TerminalKey::CtrlK => {
                buffer.truncate(cursor_pos);
                redraw_shell_line(prompt, &buffer, cursor_pos)?;
            }
            TerminalKey::CtrlW => {
                if cursor_pos > 0 {
                    let before = &buffer[..cursor_pos];
                    let trimmed = before.trim_end();
                    let word_start = trimmed.rfind(char::is_whitespace).map(|i| i + 1).unwrap_or(0);
                    buffer.drain(word_start..cursor_pos);
                    cursor_pos = word_start;
                    redraw_shell_line(prompt, &buffer, cursor_pos)?;
                }
            }
            TerminalKey::CtrlL => {
                print!("\x1b[2J\x1b[H");
                redraw_shell_line(prompt, &buffer, cursor_pos)?;
            }
            TerminalKey::Char(ch) => {
                buffer.insert(cursor_pos, ch);
                cursor_pos += ch.len_utf8();
                redraw_shell_line(prompt, &buffer, cursor_pos)?;
            }
            TerminalKey::Other => {}
        }
    }
}

// trace:FR-18 | ai:antigravity
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum ShellExitAction {
    Exit,
    Menu,
}

// trace:FR-18 | ai:antigravity
#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum ShellCommandAction {
    Continue(Vec<String>),
    Exit,
    Menu,
}

// trace:FR-18 | ai:antigravity
pub(crate) fn evaluate_shell_command(input: &str) -> Result<Option<ShellCommandAction>> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if matches!(trimmed, "exit" | "quit") {
        return Ok(Some(ShellCommandAction::Exit));
    }
    if trimmed == "menu" {
        return Ok(Some(ShellCommandAction::Menu));
    }
    let args = parse_aida_args(trimmed)?;
    Ok(Some(ShellCommandAction::Continue(args)))
}

// trace:FR-15,FR-17,FR-18 | ai:codex,antigravity
fn run_aida_shell(workspace: &Path) -> Result<ShellExitAction> {
    println!();
    println!("{}", "AIDA shell — type `exit` or `menu` to return".cyan().bold());
    let prompt = "\x1b[1;38;2;225;175;115maida>\x1b[0m ";
    let mut history = ShellHistory::new();
    loop {
        let line = read_aida_shell_line(prompt, &mut history)?;
        let Some(input) = line else {
            return Ok(ShellExitAction::Exit);
        };
        let Some(action) = evaluate_shell_command(&input)? else {
            continue;
        };
        match action {
            ShellCommandAction::Exit => return Ok(ShellExitAction::Exit),
            ShellCommandAction::Menu => return Ok(ShellExitAction::Menu),
            ShellCommandAction::Continue(args) => {
                history.push(input.trim().to_string());
                if !args.is_empty() {
                    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
                    run_aida_command(workspace, &refs)?;
                }
            }
        }
    }
}

fn parse_aida_args(input: &str) -> Result<Vec<String>> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    let mut escaped = false;
    for ch in input.chars() {
        if escaped {
            current.push(ch);
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if let Some(delimiter) = quote {
            if ch == delimiter {
                quote = None;
            } else {
                current.push(ch);
            }
        } else if ch == '\'' || ch == '"' {
            quote = Some(ch);
        } else if ch.is_whitespace() {
            if !current.is_empty() {
                args.push(std::mem::take(&mut current));
            }
        } else {
            current.push(ch);
        }
    }
    if escaped {
        current.push('\\');
    }
    if let Some(delimiter) = quote {
        bail!("unterminated {} quote", delimiter);
    }
    if !current.is_empty() {
        args.push(current);
    }
    Ok(args)
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
        mark_onboarding_completed(repo_root);
        println!(
            "{} The onboarding round trip is complete.",
            "✓".green().bold()
        );
        match mode {
            GuidanceMode::TopLevel => println!(
                "  {}",
                "Browse the full 41-exercise track with `cargo run -- list`.".dimmed()
            ),
            GuidanceMode::Shell { .. } => println!(
                "  {}",
                "Type `menu` to choose another tutorial, `lesson` to review, or `exit` to leave."
                    .dimmed()
            ),
        }
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
    let shell_run_action = matches!(mode, GuidanceMode::Shell { .. }) && nudge.starts_with("Run:");
    let displayed_nudge = if shell_run_action {
        nudge.replacen("Run:", "Type `run` or copy and paste:", 1)
    } else {
        nudge.clone()
    };
    for line in displayed_nudge.lines() {
        println!("  {}", line);
    }
    if !nudge.contains("cargo run -- next") && !shell_run_action {
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
    let mut clear_screen = true;
    run_tutorial_menu(
        workspace,
        repo_root,
        GuidanceMode::Shell { cwd: &cwd },
        &mut clear_screen,
    )?;
    let mut last_passed = current_passed(workspace)?;

    println!();
    println!(
        "{}",
        "Interactive shell commands: next, run, menu, lesson, where, reset, help, exit. Other input runs in your shell.".dimmed()
    );
    println!();

    let stdin = io::stdin();
    let mut framed_prompt = true;
    loop {
        if framed_prompt {
            print_shell_input_start(repo_root, &cwd);
        } else {
            print_shell_plain_prompt(repo_root, &cwd);
        }
        io::stdout().flush()?;

        let mut line = String::new();
        if stdin.read_line(&mut line)? == 0 {
            if framed_prompt {
                print_shell_input_end();
            } else {
                print_shell_input_reset();
            }
            println!();
            break;
        }
        if framed_prompt {
            print_shell_input_end();
        } else {
            print_shell_input_reset();
        }
        let input = line.trim();
        if input.is_empty() {
            framed_prompt = false;
            continue;
        }
        framed_prompt = true;

        if let Some(target) = input
            .strip_prefix("exercise ")
            .or_else(|| input.strip_prefix("show "))
            .map(str::trim)
            .filter(|target| !target.is_empty())
        {
            run_tutor_subcommand(repo_root, &["show", target])?;
            continue;
        }
        if input == "verify" {
            run_tutor_subcommand(repo_root, &["verify"])?;
            continue;
        }
        if let Some(target) = input
            .strip_prefix("verify ")
            .map(str::trim)
            .filter(|target| !target.is_empty())
        {
            run_tutor_subcommand(repo_root, &["verify", target])?;
            continue;
        }

        match input {
            "exit" | "quit" => break,
            // trace:FR-11 | ai:codex
            "help" | "?" => {
                print_shell_help();
                continue;
            }
            "next" => {
                last_passed =
                    print_next_state(workspace, repo_root, GuidanceMode::Shell { cwd: &cwd })?;
                continue;
            }
            // trace:FR-5 | ai:codex
            "menu" | "tutorials" => {
                run_tutorial_menu(
                    workspace,
                    repo_root,
                    GuidanceMode::Shell { cwd: &cwd },
                    &mut clear_screen,
                )?;
                last_passed = current_passed(workspace)?;
                continue;
            }
            // trace:FR-5 | ai:codex
            "onboarding" | "lesson" => {
                run(workspace, repo_root, false)?;
                last_passed = current_passed(workspace)?;
                continue;
            }
            // trace:FR-5 | ai:codex
            "exercises" => {
                run_exercise_menu(workspace, repo_root)?;
                continue;
            }
            // trace:FR-3 | ai:codex
            "run" => {
                let selection = select_or_load_agents(workspace)?;
                let cps = checkpoints(&selection);
                let passed = passed_count(workspace, &cps, &selection);
                if passed >= cps.len() {
                    println!(
                        "{} The onboarding round trip is already complete.",
                        "✓".green()
                    );
                    continue;
                }
                let nudge = checkpoint_nudge(
                    workspace,
                    &cps,
                    passed,
                    &selection,
                    GuidanceMode::Shell { cwd: &cwd },
                );
                match runnable_next_command(workspace, passed, &selection, &nudge) {
                    Some(command) => {
                        println!("{} {}", "$".dimmed(), command);
                        let status = Command::new("sh")
                            .arg("-c")
                            .arg(&command)
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
                        let current = current_passed(workspace)?;
                        if current > passed {
                            announce_progress(workspace, repo_root, passed, current, &cwd)?;
                        } else if status.success() {
                            announce_current_action(workspace, current, &cwd)?;
                        }
                        last_passed = current;
                    }
                    None => println!(
                        "{} The current step needs a human decision or manual input. Type `next` for guidance.",
                        "•".yellow()
                    ),
                }
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

// trace:FR-4 | ai:codex
fn print_shell_input_start(repo_root: &Path, cwd: &Path) {
    println!();
    println!("{}", "─".repeat(SHELL_RULE_WIDTH).dimmed());
    print!(
        "\x1b[48;5;236m\x1b[36;1m{}\x1b[0m\x1b[48;5;236m ❯ \x1b[K",
        shell_prompt(repo_root, cwd)
    );
}

// trace:FR-4 | ai:codex
fn print_shell_plain_prompt(repo_root: &Path, cwd: &Path) {
    print!(
        "\x1b[48;5;236m\x1b[36;1m{}\x1b[0m\x1b[48;5;236m ❯ \x1b[K",
        shell_prompt(repo_root, cwd)
    );
}

// trace:FR-4 | ai:codex
fn print_shell_input_end() {
    // The reset is deliberately emitted after read_line: the terminal echoes
    // the learner's command inside the dark input band.
    print!("\x1b[0m\n{}\n\n", "─".repeat(SHELL_RULE_WIDTH).dimmed());
}

fn print_shell_input_reset() {
    print!("\x1b[0m");
}

fn print_shell_help() {
    println!("{}", "Built-ins:".bold());
    println!("  next    show the current onboarding step and next action");
    println!("  run     execute the current machine-runnable next action");
    println!("  menu    show the available tutorial paths");
    println!("  exercises launch the full exercise track");
    println!("  exercise N open exercise N");
    println!("  verify N record exercise N after it passes");
    println!("  ?       alias for help");
    println!("  lesson  render the full current onboarding lesson");
    println!("  where   print the shell's current directory");
    println!("  reset   reset workspace/ and restart onboarding");
    println!("  cd DIR  change the shell's current directory");
    println!("  exit    leave the tutorial shell");
    println!();
    println!("Other input is executed by `sh -c` from the current directory.");
}

struct TerminalMode {
    saved: String,
}

impl TerminalMode {
    fn enter() -> Result<Self> {
        let saved = String::from_utf8(
            Command::new("stty")
                .arg("-g")
                .output()
                .context("reading terminal mode")?
                .stdout,
        )?
        .trim()
        .to_owned();
        let status = Command::new("stty")
            .args(["-icanon", "-echo"])
            .status()
            .context("enabling tutorial menu input")?;
        if !status.success() {
            bail!("could not enable interactive tutorial menu input");
        }
        Ok(Self { saved })
    }
}

impl Drop for TerminalMode {
    fn drop(&mut self) {
        if self.saved.is_empty() {
            let _ = Command::new("stty").arg("sane").status();
        } else {
            let _ = Command::new("stty").arg(&self.saved).status();
        }
    }
}

// trace:BUG-15 | ai:antigravity
fn mark_onboarding_completed(repo_root: &Path) {
    if let Ok(mut prog) = Progress::load(repo_root) {
        if !prog.is_onboarding_completed() {
            prog.record_onboarding_completion();
            let _ = prog.save(repo_root);
        }
    }
}

// trace:BUG-15 | ai:antigravity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OnboardingStatus {
    NotStarted,
    InProgress { passed: usize },
    Completed,
}

// trace:BUG-15 | ai:antigravity
pub(crate) fn onboarding_status(workspace: &Path, repo_root: &Path) -> OnboardingStatus {
    if let Ok(prog) = Progress::load(repo_root) {
        if prog.is_onboarding_completed() {
            return OnboardingStatus::Completed;
        }
    }

    let Some(selection) = load_agent_selection(workspace).or_else(|| {
        infer_agent_choice_from_trace(workspace)
            .map(|choice| AgentSelection::new(choice, vec![choice]))
    }) else {
        if fr_spec_id(workspace).is_none() && verify::trace_comments_in_workspace(workspace).is_empty() {
            return OnboardingStatus::NotStarted;
        }
        return OnboardingStatus::InProgress { passed: 1 };
    };

    let cps = checkpoints(&selection);
    let passed = passed_count(workspace, &cps, &selection);
    if passed >= cps.len() {
        mark_onboarding_completed(repo_root);
        OnboardingStatus::Completed
    } else if passed > 0 || selected_agent_exists(workspace) {
        OnboardingStatus::InProgress { passed }
    } else {
        OnboardingStatus::NotStarted
    }
}

// trace:BUG-15 | ai:antigravity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TutorialOption {
    Resume,
    Repeat,
    Onboarding,
    Exercises,
    Terminology,
    Faq,
    Exit,
}

impl TutorialOption {
    pub(crate) fn label(&self, passed: usize, exit_desc: &str) -> String {
        match self {
            TutorialOption::Resume => {
                if passed > 0 {
                    format!("resume      — show current onboarding action (Step {passed} of 5)")
                } else {
                    "resume      — show current onboarding action".to_string()
                }
            }
            TutorialOption::Repeat => "repeat      — restart onboarding from the beginning".to_string(),
            TutorialOption::Onboarding => "onboarding  — the 15-minute capture → trace → commit round trip".to_string(),
            TutorialOption::Exercises => "exercises   — the full 41-exercise AIDA workflow track".to_string(),
            TutorialOption::Terminology => "terminology — a glossary of AIDA concepts".to_string(),
            TutorialOption::Faq => "FAQ         — common questions and practical answers".to_string(),
            TutorialOption::Exit => format!("exit        — {exit_desc}"),
        }
    }
}

// trace:BUG-15 | ai:antigravity
pub(crate) fn tutorial_options(status: OnboardingStatus) -> Vec<TutorialOption> {
    match status {
        OnboardingStatus::NotStarted => vec![
            TutorialOption::Onboarding,
            TutorialOption::Exercises,
            TutorialOption::Terminology,
            TutorialOption::Faq,
            TutorialOption::Exit,
        ],
        OnboardingStatus::InProgress { .. } => vec![
            TutorialOption::Resume,
            TutorialOption::Onboarding,
            TutorialOption::Exercises,
            TutorialOption::Terminology,
            TutorialOption::Faq,
            TutorialOption::Exit,
        ],
        OnboardingStatus::Completed => vec![
            TutorialOption::Repeat,
            TutorialOption::Onboarding,
            TutorialOption::Exercises,
            TutorialOption::Terminology,
            TutorialOption::Faq,
            TutorialOption::Exit,
        ],
    }
}

// trace:FR-6,BUG-15,FR-21 | ai:codex,antigravity
fn run_tutorial_menu(
    workspace: &Path,
    repo_root: &Path,
    mode: GuidanceMode<'_>,
    clear_screen: &mut bool,
) -> Result<()> {
    let interactive = io::stdin().is_terminal();
    if interactive && *clear_screen {
        print!("\x1b[2J\x1b[H");
    }
    let status = onboarding_status(workspace, repo_root);
    let options = tutorial_options(status);
    let passed = match status {
        OnboardingStatus::InProgress { passed } => passed,
        _ => 0,
    };
    let exit_desc = match mode {
        GuidanceMode::TopLevel => "return to the menu",
        GuidanceMode::Shell { .. } => "return to the shell",
    };

    if !interactive {
        println!();
        println!("{}", "Help / Tutorials".cyan().bold());
        for (i, opt) in options.iter().enumerate() {
            println!("  {}. {}", i + 1, opt.label(passed, exit_desc));
        }
        println!("  Use `menu` from an interactive terminal to select with arrows.");
        return Ok(());
    }

    let _terminal_mode = TerminalMode::enter()?;
    let mut selected = 0usize;
    let max_idx = options.len() - 1;
    draw_tutorial_menu(&options, selected, false, passed, exit_desc, *clear_screen);
    let mut stdin = io::stdin();
    let mut byte = [0u8; 1];
    loop {
        if !read_terminal_byte(&stdin, &mut byte[0]) {
            break;
        }
        match byte[0] {
            b'\r' | b'\n' => break,
            3 => {
                println!();
                return Ok(());
            }
            b'c' | b'C' => {
                *clear_screen = !*clear_screen;
                draw_tutorial_menu(&options, selected, true, passed, exit_desc, *clear_screen);
            }
            27 => match read_escape_sequence(&mut stdin) {
                Some([b'[', b'A']) => {
                    selected = selected.saturating_sub(1);
                    draw_tutorial_menu(&options, selected, true, passed, exit_desc, *clear_screen);
                }
                Some([b'[', b'B']) => {
                    selected = (selected + 1).min(max_idx);
                    draw_tutorial_menu(&options, selected, true, passed, exit_desc, *clear_screen);
                }
                _ => return Ok(()),
            },
            b'k' => {
                selected = selected.saturating_sub(1);
                draw_tutorial_menu(&options, selected, true, passed, exit_desc, *clear_screen);
            }
            b'j' => {
                selected = (selected + 1).min(max_idx);
                draw_tutorial_menu(&options, selected, true, passed, exit_desc, *clear_screen);
            }
            _ => continue,
        }
    }
    println!();
    drop(_terminal_mode);

    let pause_msg = match mode {
        GuidanceMode::TopLevel => "Press Enter to return to menu...",
        GuidanceMode::Shell { .. } => "Press Enter to return to shell...",
    };

    match options[selected] {
        TutorialOption::Resume => {
            if let Err(e) = print_next_state(workspace, repo_root, mode) {
                println!("{} {e}", "Error:".red().bold());
                pause_for_enter();
                return Err(e);
            }
            pause_for_enter_prompt(pause_msg);
            Ok(())
        }
        TutorialOption::Repeat => {
            if let Err(e) = run(workspace, repo_root, true) {
                println!("{} {e}", "Error:".red().bold());
                pause_for_enter();
                return Err(e);
            }
            pause_for_enter_prompt(pause_msg);
            Ok(())
        }
        TutorialOption::Onboarding => {
            if let Err(e) = run(workspace, repo_root, false) {
                println!("{} {e}", "Error:".red().bold());
                pause_for_enter();
                return Err(e);
            }
            pause_for_enter_prompt(pause_msg);
            Ok(())
        }
        TutorialOption::Exercises => {
            if let Err(e) = run_exercise_menu(workspace, repo_root) {
                println!("{} {e}", "Error:".red().bold());
                pause_for_enter();
                return Err(e);
            }
            Ok(())
        }
        TutorialOption::Terminology => {
            run_terminology_menu(repo_root)?;
            run_tutorial_menu(workspace, repo_root, mode, clear_screen)
        }
        TutorialOption::Faq => {
            run_faq_menu(repo_root)?;
            run_tutorial_menu(workspace, repo_root, mode, clear_screen)
        }
        TutorialOption::Exit => Ok(()),
    }
}

// trace:BUG-15,FR-21 | ai:antigravity
fn draw_tutorial_menu(
    options: &[TutorialOption],
    selected: usize,
    redraw: bool,
    passed: usize,
    exit_desc: &str,
    clear_screen: bool,
) {
    if redraw {
        let lines_up = options.len() + 3;
        print!("\x1b[{}A\r", lines_up);
    }
    println!("{}", "Help / Tutorials".cyan().bold());
    println!();
    for (i, opt) in options.iter().enumerate() {
        let cursor = if i == selected { "❯" } else { " " };
        println!("{cursor} {}", opt.label(passed, exit_desc));
    }
    let clear_status = if clear_screen { "on" } else { "off" };
    println!("  Use ↑/↓ (or j/k), Enter to select, c to toggle clear [{clear_status}], Esc to cancel.");
    print!("\x1b[J");
    let _ = io::stdout().flush();
}

#[derive(Debug, Deserialize)]
struct JsonTerm {
    id: String,
    term: String,
    category: String,
    summary: String,
    detail: String,
    tags: Vec<String>,
    see_also: Vec<String>,
    refs: Vec<String>,
    #[serde(default)]
    examples: Vec<String>,
    #[serde(default)]
    when_to_use: Option<String>,
    #[serde(default)]
    used_by: Vec<String>,
    #[serde(default)]
    best_when: Option<String>,
    #[serde(default)]
    pros: Vec<String>,
    #[serde(default)]
    cons: Vec<String>,
}

// trace:FR-8 | ai:codex
fn run_terminology_menu(repo_root: &Path) -> Result<()> {
    let terms = load_terminology(repo_root)?;
    if !io::stdin().is_terminal() {
        println!();
        println!("{}", "Terminology".cyan().bold());
        for term in &terms {
            println!("  {:<20} — {}", term.term, term.summary);
        }
        return Ok(());
    }

    let _terminal_mode = TerminalMode::enter()?;
    let mut selected = 0usize;
    let mut stdin = io::stdin();
    let mut byte = [0u8; 1];
    loop {
        draw_terminology_menu(&terms, selected);
        if !read_terminal_byte(&stdin, &mut byte[0]) {
            return Ok(());
        }
        match byte[0] {
            b'\r' | b'\n' => {
                selected = browse_term_detail(&terms, selected, &mut stdin)?;
            }
            3 => return Ok(()),
            b'k' => selected = selected.saturating_sub(1),
            b'j' => selected = (selected + 1).min(terms.len() - 1),
            0x1b => match read_escape_sequence(&mut stdin) {
                Some([b'[', b'A']) => selected = selected.saturating_sub(1),
                Some([b'[', b'B']) => selected = (selected + 1).min(terms.len() - 1),
                Some([b'[', b'C']) => selected = browse_term_detail(&terms, selected, &mut stdin)?,
                Some([b'[', b'D']) | None => return Ok(()),
                _ => {}
            },
            _ => {}
        }
    }
}

// trace:FR-13 | ai:codex
fn browse_term_detail(
    terms: &[JsonTerm],
    mut selected: usize,
    stdin: &mut io::Stdin,
) -> Result<usize> {
    let mut related_selected = 0usize;
    let mut byte = [0u8; 1];
    loop {
        let term = &terms[selected];
        print!("\x1b[2J\x1b[H");
        println!("{}", term.term.cyan().bold());
        println!("{} · {}", term.category, term.id);
        println!();
        println!("{}", term.detail);
        println!();
        println!("Tags: {}", term.tags.join(", "));
        if let Some(when_to_use) = &term.when_to_use {
            println!("When to use: {when_to_use}");
        }
        if let Some(best_when) = &term.best_when {
            println!("Best when: {best_when}");
        }
        if !term.used_by.is_empty() {
            println!("Used by: {}", term.used_by.join(", "));
        }
        if !term.examples.is_empty() {
            println!("Examples:");
            for example in &term.examples {
                println!("  • {example}");
            }
        }
        if !term.pros.is_empty() {
            println!("Pros: {}", term.pros.join("; "));
        }
        if !term.cons.is_empty() {
            println!("Cons: {}", term.cons.join("; "));
        }
        println!();
        println!("See also:");
        for (index, related_id) in term.see_also.iter().enumerate() {
            let label = terms
                .iter()
                .find(|candidate| candidate.id == *related_id)
                .map(|candidate| candidate.term.as_str())
                .unwrap_or(related_id.as_str());
            println!(
                "  {} {}",
                if index == related_selected {
                    "❯"
                } else {
                    " "
                },
                label
            );
        }
        println!("References: {}", term.refs.join(", "));
        println!();
        println!("↑/↓ select a related term · ← return · → next term · Enter open · Esc return");
        let _ = io::stdout().flush();

        if !read_terminal_byte(stdin, &mut byte[0]) {
            return Ok(selected);
        }
        match byte[0] {
            b'\r' | b'\n' => {
                if let Some(related_id) = term.see_also.get(related_selected) {
                    if let Some(next) = terms
                        .iter()
                        .position(|candidate| candidate.id == *related_id)
                    {
                        selected = next;
                        related_selected = 0;
                    }
                }
            }
            3 => return Ok(selected),
            b'k' => related_selected = related_selected.saturating_sub(1),
            b'j' => {
                if !term.see_also.is_empty() {
                    related_selected = (related_selected + 1).min(term.see_also.len() - 1);
                }
            }
            0x1b => match read_escape_sequence(stdin) {
                Some([b'[', b'A']) => related_selected = related_selected.saturating_sub(1),
                Some([b'[', b'B']) if !term.see_also.is_empty() => {
                    related_selected = (related_selected + 1).min(term.see_also.len() - 1)
                }
                Some([b'[', b'C']) => selected = (selected + 1).min(terms.len() - 1),
                Some([b'[', b'D']) | None => return Ok(selected),
                _ => {}
            },
            _ => {}
        }
    }
}

/// Read the two bytes following an Esc only when they are actually available.
/// A bare Esc is a navigation key, not the beginning of a malformed arrow
/// sequence. trace:FR-13 | ai:codex
fn read_escape_sequence(stdin: &mut io::Stdin) -> Option<[u8; 2]> {
    let fd = stdin.as_raw_fd();
    let mut sequence = [0u8; 2];
    for byte in &mut sequence {
        let mut pollfd = libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        };
        let ready = unsafe { libc::poll(&mut pollfd, 1, 75) };
        if ready <= 0 || pollfd.revents & libc::POLLIN == 0 {
            return None;
        }
        if !read_terminal_byte(stdin, byte) {
            return None;
        }
    }
    Some(sequence)
}

fn read_terminal_byte(stdin: &io::Stdin, byte: &mut u8) -> bool {
    unsafe { libc::read(stdin.as_raw_fd(), byte as *mut u8 as *mut libc::c_void, 1) == 1 }
}

fn load_terminology(repo_root: &Path) -> Result<Vec<JsonTerm>> {
    let path = repo_root.join("content/terminology.json");
    let contents = std::fs::read_to_string(&path)
        .with_context(|| format!("reading terminology file {}", path.display()))?;
    let terms: Vec<JsonTerm> = serde_json::from_str(&contents)
        .with_context(|| format!("parsing terminology file {}", path.display()))?;
    validate_terminology(&terms, repo_root)?;
    Ok(terms)
}

fn validate_terminology(terms: &[JsonTerm], repo_root: &Path) -> Result<()> {
    let ids: std::collections::HashSet<&str> = terms.iter().map(|term| term.id.as_str()).collect();
    if ids.len() != terms.len() {
        bail!("terminology contains duplicate ids");
    }
    for term in terms {
        for related in &term.see_also {
            if !ids.contains(related.as_str()) {
                bail!(
                    "terminology {} references unknown term {}",
                    term.id,
                    related
                );
            }
        }
        for reference in &term.refs {
            if !repo_root.join(reference).exists() {
                bail!(
                    "terminology {} references missing file {}",
                    term.id,
                    reference
                );
            }
        }
    }
    Ok(())
}

fn draw_terminology_menu(terms: &[JsonTerm], selected: usize) {
    print!("\x1b[2J\x1b[H");
    println!("{}", "Terminology".cyan().bold());
    println!("Use ↑/↓ (or j/k), Enter to expand, Esc to return to the tutorial menu.\n");
    for (index, term) in terms.iter().enumerate() {
        println!(
            "{} {:<20} — {}",
            if index == selected { "❯" } else { " " },
            term.term,
            term.summary
        );
    }
    let _ = io::stdout().flush();
}

#[derive(Debug, Deserialize)]
struct FaqEntry {
    id: String,
    question: String,
    answer: String,
    detail: String,
    path: Vec<String>,
    tags: Vec<String>,
    see_also: Vec<String>,
    refs: Vec<String>,
}

// trace:FR-12 | ai:codex
fn run_faq_menu(repo_root: &Path) -> Result<()> {
    let entries = load_faq(repo_root)?;
    if !io::stdin().is_terminal() {
        println!();
        println!("{}", "FAQ".cyan().bold());
        for entry in &entries {
            println!(
                "  {} / {} — {}",
                entry.path.join(" / "),
                entry.question,
                entry.answer
            );
        }
        return Ok(());
    }

    let _terminal_mode = TerminalMode::enter()?;
    let mut prefix = Vec::new();
    let mut selected = 0usize;
    let mut stdin = io::stdin();
    let mut byte = [0u8; 1];
    loop {
        let items = faq_items(&entries, &prefix);
        if items.is_empty() {
            return Ok(());
        }
        selected = selected.min(items.len() - 1);
        draw_faq_menu(&entries, &prefix, &items, selected);
        if !read_terminal_byte(&stdin, &mut byte[0]) {
            return Ok(());
        }
        match byte[0] {
            b'\r' | b'\n' => match items[selected] {
                FaqItem::Category(ref label) => {
                    prefix.push(label.clone());
                    selected = 0;
                }
                FaqItem::Question(index) => {
                    draw_faq_detail(&entries[index], &mut stdin, &mut byte);
                }
            },
            3 => return Ok(()),
            b'k' => selected = selected.saturating_sub(1),
            b'j' => selected = (selected + 1).min(items.len() - 1),
            0x1b => match read_escape_sequence(&mut stdin) {
                Some([b'[', b'A']) => selected = selected.saturating_sub(1),
                Some([b'[', b'B']) => selected = (selected + 1).min(items.len() - 1),
                _ if prefix.pop().is_some() => selected = 0,
                _ => return Ok(()),
            },
            _ => {}
        }
    }
}

enum FaqItem {
    Category(String),
    Question(usize),
}

// trace:FR-14 | ai:codex
fn faq_items(entries: &[FaqEntry], prefix: &[String]) -> Vec<FaqItem> {
    let mut items = Vec::new();
    let mut categories = std::collections::HashSet::new();
    for (index, entry) in entries.iter().enumerate() {
        if entry.path.len() < prefix.len() || entry.path[..prefix.len()] != *prefix {
            continue;
        }
        if entry.path.len() == prefix.len() {
            items.push(FaqItem::Question(index));
        } else if categories.insert(entry.path[prefix.len()].clone()) {
            items.push(FaqItem::Category(entry.path[prefix.len()].clone()));
        }
    }
    items
}

fn draw_faq_detail(entry: &FaqEntry, stdin: &mut io::Stdin, byte: &mut [u8; 1]) {
    print!("\x1b[2J\x1b[H");
    println!("{}", entry.question.cyan().bold());
    println!("{} · {}", entry.path.join(" / "), entry.id);
    println!();
    println!("{}", entry.answer);
    println!();
    println!("{}", entry.detail);
    println!();
    println!("Tags: {}", entry.tags.join(", "));
    println!("See also: {}", entry.see_also.join(", "));
    println!("References: {}", entry.refs.join(", "));
    println!();
    println!("Press Esc to return to this FAQ topic.");
    let _ = io::stdout().flush();
    let _ = read_terminal_byte(stdin, &mut byte[0]);
}

fn load_faq(repo_root: &Path) -> Result<Vec<FaqEntry>> {
    let path = repo_root.join("content/faq.json");
    let contents = std::fs::read_to_string(&path)
        .with_context(|| format!("reading FAQ file {}", path.display()))?;
    let entries: Vec<FaqEntry> = serde_json::from_str(&contents)
        .with_context(|| format!("parsing FAQ file {}", path.display()))?;
    validate_faq(&entries, repo_root)?;
    Ok(entries)
}

fn validate_faq(entries: &[FaqEntry], repo_root: &Path) -> Result<()> {
    let ids: std::collections::HashSet<&str> =
        entries.iter().map(|entry| entry.id.as_str()).collect();
    if ids.len() != entries.len() {
        bail!("FAQ contains duplicate ids");
    }
    for entry in entries {
        if entry.path.is_empty() {
            bail!("FAQ {} must have at least one path component", entry.id);
        }
        for related in &entry.see_also {
            if !ids.contains(related.as_str()) {
                bail!("FAQ {} references unknown entry {}", entry.id, related);
            }
        }
        for reference in &entry.refs {
            if !repo_root.join(reference).exists() {
                bail!("FAQ {} references missing file {}", entry.id, reference);
            }
        }
    }
    Ok(())
}

fn draw_faq_menu(entries: &[FaqEntry], prefix: &[String], items: &[FaqItem], selected: usize) {
    print!("\x1b[2J\x1b[H");
    let title = if prefix.is_empty() {
        "FAQ".to_string()
    } else {
        format!("FAQ / {}", prefix.join(" / "))
    };
    println!("{}", title.cyan().bold());
    let escape_destination = if prefix.is_empty() {
        "the tutorial menu"
    } else {
        "the parent topic"
    };
    println!(
        "Use ↑/↓ (or j/k), Enter to open, Esc to return to {}.\n",
        escape_destination
    );
    for (index, item) in items.iter().enumerate() {
        let marker = if index == selected { "❯" } else { " " };
        match item {
            FaqItem::Category(label) => {
                let count = entries
                    .iter()
                    .filter(|entry| {
                        entry.path.len() > prefix.len()
                            && entry.path[..prefix.len()] == *prefix
                            && entry.path[prefix.len()] == *label
                    })
                    .count();
                println!("{} {} / ({count} questions)", marker, label);
            }
            FaqItem::Question(entry_index) => {
                println!(
                    "{} ? {} — {}",
                    marker, entries[*entry_index].question, entries[*entry_index].answer
                );
            }
        }
    }
    let _ = io::stdout().flush();
}

// trace:FR-6 | ai:codex
fn run_exercise_menu(workspace: &Path, repo_root: &Path) -> Result<()> {
    let exercises = crate::exercises::all();
    let progress = refresh_exercise_progress(&exercises, workspace, repo_root)?;
    if !io::stdin().is_terminal() {
        run_tutor_subcommand(repo_root, &["list"])?;
        return Ok(());
    }

    let _terminal_mode = TerminalMode::enter()?;
    let mut selected = progress
        .current(exercises.len() as u32)
        .unwrap_or(1)
        .saturating_sub(1) as usize;
    let mut stdin = io::stdin();
    let mut byte = [0u8; 1];
    loop {
        draw_exercise_menu(&exercises, workspace, &progress, selected);
        if !read_terminal_byte(&stdin, &mut byte[0]) {
            return Ok(());
        }
        match byte[0] {
            b'\r' | b'\n' => break,
            3 => return Ok(()),
            27 => match read_escape_sequence(&mut stdin) {
                Some([b'[', b'A']) => selected = selected.saturating_sub(1),
                Some([b'[', b'B']) => selected = (selected + 1).min(exercises.len() - 1),
                _ => return Ok(()),
            },
            b'k' => selected = selected.saturating_sub(1),
            b'j' => selected = (selected + 1).min(exercises.len() - 1),
            _ => {}
        }
    }
    println!();
    let id = exercises[selected].id().to_string();
    run_tutor_subcommand(repo_root, &["show", &id])?;
    pause_for_enter_prompt("Press Enter to return to menu...");
    Ok(())
}

// trace:FR-7 | ai:codex
fn refresh_exercise_progress(
    exercises: &[Box<dyn crate::exercise::Exercise>],
    workspace: &Path,
    repo_root: &Path,
) -> Result<Progress> {
    let mut progress = Progress::load(repo_root)?;
    let mut recorded = Vec::new();
    for exercise in exercises {
        if !progress.is_completed(exercise.id())
            && matches!(exercise.verify(workspace), VerifyResult::Pass)
        {
            progress.record_completion(exercise.id());
            recorded.push(exercise.id());
            if exercise.id() == 19 && progress.store_commit_baseline.is_none() {
                progress.store_commit_baseline =
                    crate::verify::git_commit_count(workspace, "aida-store");
            }
        }
    }
    if !recorded.is_empty() {
        progress.save(repo_root)?;
        println!(
            "{} Menu refreshed progress for exercise(s): {}",
            "✓".green(),
            recorded
                .iter()
                .map(|id| format!("{id:02}"))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    Ok(progress)
}

fn draw_exercise_menu(
    exercises: &[Box<dyn crate::exercise::Exercise>],
    workspace: &Path,
    progress: &Progress,
    selected: usize,
) {
    print!("\x1b[2J\x1b[H");
    println!("{}", "Exercises".cyan().bold());
    println!("Use ↑/↓ (or j/k), Enter to open, Esc to return.\n");
    for (index, exercise) in exercises.iter().enumerate() {
        let marker = if index == selected { "❯" } else { " " };
        let state = if progress.is_completed(exercise.id()) {
            "completed".green().to_string()
        } else {
            match exercise.verify(workspace) {
                VerifyResult::Pass => "ready — verify to record".yellow().to_string(),
                VerifyResult::Pending(_) => "not started".dimmed().to_string(),
                VerifyResult::Fail(_) => "needs work".red().to_string(),
            }
        };
        println!(
            "{} {:>2}. {:<48} [{}]",
            marker,
            exercise.id(),
            exercise.title(),
            state
        );
    }
    let _ = io::stdout().flush();
}

// trace:FR-5,FR-21 | ai:codex,antigravity
fn run_tutor_subcommand(repo_root: &Path, args: &[&str]) -> Result<()> {
    let status = Command::new("cargo")
        .arg("run")
        .arg("--")
        .args(args)
        .current_dir(repo_root)
        .status()
        .with_context(|| "launching an exercise command")?;
    if !status.success() {
        println!(
            "{} exercise command exited with status {}",
            "✗".red().bold(),
            status.code().unwrap_or(1)
        );
        pause_for_enter();
    }
    Ok(())
}

// trace:FR-3 | ai:codex
fn runnable_next_command(
    workspace: &Path,
    passed: usize,
    selection: &AgentSelection,
    nudge: &str,
) -> Option<String> {
    if passed == 0 && !verify::is_aida_initialized(workspace) {
        return Some(selection.init_command().to_owned());
    }

    nudge.lines().map(str::trim).find_map(|line| {
        let runnable = ["aida ", "codex ", "claude ", "cargo ", "git ", "python "]
            .iter()
            .any(|prefix| line.starts_with(prefix));
        runnable.then(|| line.to_owned())
    })
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
            "Browse the full 41-exercise track any time with `aida-tutor list`.".dimmed()
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

    if passed == 3 {
        let prefix = shell_command_prefix(workspace, mode);
        let suffix = command_suffix(next_followup(mode));
        return format!(
            "Run:\n\n    {}git add greet.py && git commit -m {}{}",
            prefix,
            shell_single_quote(&format!(
                "{}(greet): add --upper flag (FR-1)",
                selection
                    .primary
                    .commit_prefix()
                    .replace("{{trace_tool}}", selection.primary.trace_tool())
            )),
            suffix
        );
    }

    if passed == 4 {
        let prefix = shell_command_prefix(workspace, mode);
        let suffix = command_suffix(next_followup(mode));
        return format!(
            "Run:\n\n    {}aida edit FR-1 --status completed{}",
            prefix, suffix
        );
    }

    cps[passed].nudge.clone()
}

// trace:BUG-14 | ai:codex
fn commit_nudge(selection: &AgentSelection) -> String {
    format!(
        "Run:\n\n    git add greet.py && git commit -m {}",
        shell_single_quote(&format!(
            "{}(greet): add --upper flag (FR-1)",
            selection
                .primary
                .commit_prefix()
                .replace("{{trace_tool}}", selection.primary.trace_tool())
        ))
    )
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
    fn implementation_readiness_treats_parent_only_as_tour_safe() {
        let output = "  ✓ priority — priority is set\n  ✗ parent — no parent link\n";
        assert_eq!(
            implementation_readiness_from_output(output),
            ImplementationReadiness::StandaloneParentOnly
        );
    }

    #[test]
    fn implementation_readiness_preserves_real_blockers() {
        let output = "  ✗ parent — no parent link\n  ✗ acceptance — missing acceptance\n";
        assert_eq!(
            implementation_readiness_from_output(output),
            ImplementationReadiness::Blocked(
                "parent — no parent link; acceptance — missing acceptance".to_owned()
            )
        );
    }

    #[test]
    fn run_extracts_machine_action_without_executing_prose() {
        let ws = tmp();
        let selection = selection(AgentChoice::Codex, &[AgentChoice::Codex]);
        let nudge = "Readiness note: continue directly.\n\n    codex exec --cd /tmp/work --sandbox workspace-write 'do it'";

        assert_eq!(
            runnable_next_command(&ws, 2, &selection, nudge).as_deref(),
            Some("codex exec --cd /tmp/work --sandbox workspace-write 'do it'")
        );
        std::fs::remove_dir_all(&ws).ok();
    }

    #[test]
    fn commit_checkpoint_shows_copyable_spec_linked_command() {
        let ws = tmp();
        let selected = selection(AgentChoice::Codex, &[AgentChoice::Codex]);
        let cps = checkpoints(&selected);
        let nudge = checkpoint_nudge(&ws, &cps, 3, &selected, GuidanceMode::Shell { cwd: &ws });

        assert!(nudge.contains("git add greet.py && git commit"));
        assert!(nudge.contains("[AI:codex] feat(greet): add --upper flag (FR-1)"));
        assert_eq!(
            runnable_next_command(&ws, 3, &selected, &nudge).as_deref(),
            Some(
                "git add greet.py && git commit -m '[AI:codex] feat(greet): add --upper flag (FR-1)'"
            )
        );
        std::fs::remove_dir_all(&ws).ok();
    }

    #[test]
    fn closeout_checkpoint_is_runnable() {
        let ws = tmp();
        let selected = selection(AgentChoice::Codex, &[AgentChoice::Codex]);
        let cps = checkpoints(&selected);
        let nudge = checkpoint_nudge(&ws, &cps, 4, &selected, GuidanceMode::Shell { cwd: &ws });

        assert!(!nudge.contains("Type"));
        assert_eq!(
            runnable_next_command(&ws, 4, &selected, &nudge).as_deref(),
            Some("aida edit FR-1 --status completed")
        );
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
    fn faq_tree_keeps_questions_at_the_selected_leaf_path() {
        let entries = vec![FaqEntry {
            id: "faq.test".to_owned(),
            question: "How?".to_owned(),
            answer: "This way.".to_owned(),
            detail: "A detail.".to_owned(),
            path: vec!["Tutor".to_owned(), "Basics".to_owned()],
            tags: vec![],
            see_also: vec![],
            refs: vec![],
        }];

        assert!(matches!(
            faq_items(&entries, &["Tutor".to_owned()])[0],
            FaqItem::Category(ref label) if label == "Basics"
        ));
        assert!(matches!(
            faq_items(&entries, &["Tutor".to_owned(), "Basics".to_owned()])[0],
            FaqItem::Question(0)
        ));
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

    #[test]
    fn aida_shell_history_empty_navigation() {
        let mut history = ShellHistory::new();
        assert_eq!(history.navigate_up("initial"), None);
        assert_eq!(history.navigate_down(), None);
    }

    #[test]
    fn aida_shell_history_up_down_cycle() {
        let mut history = ShellHistory::new();
        history.push("aida list".to_string());
        history.push("aida show FR-1".to_string());

        // Typing draft
        assert_eq!(history.navigate_up("aida doc"), Some("aida show FR-1"));
        assert_eq!(history.navigate_up("aida show FR-1"), Some("aida list"));
        assert_eq!(history.navigate_up("aida list"), None); // top of history

        // Moving down
        assert_eq!(history.navigate_down(), Some("aida show FR-1"));
        assert_eq!(history.navigate_down(), Some("aida doc")); // draft restored!
        assert_eq!(history.navigate_down(), None); // already at draft
    }

    #[test]
    fn aida_shell_history_deduplicates_consecutive_commands() {
        let mut history = ShellHistory::new();
        history.push("status".to_string());
        history.push("status".to_string());
        history.push("status".to_string());
        history.push("list".to_string());

        assert_eq!(history.entries.len(), 2);
        assert_eq!(history.entries[0], "status");
        assert_eq!(history.entries[1], "list");
    }

    #[test]
    fn aida_shell_menu_command_returns_menu_action() {
        assert_eq!(
            evaluate_shell_command("menu").unwrap(),
            Some(ShellCommandAction::Menu)
        );
        assert_eq!(
            evaluate_shell_command("  menu  ").unwrap(),
            Some(ShellCommandAction::Menu)
        );
        assert_eq!(
            evaluate_shell_command("exit").unwrap(),
            Some(ShellCommandAction::Exit)
        );
        assert_eq!(
            evaluate_shell_command("quit").unwrap(),
            Some(ShellCommandAction::Exit)
        );
        assert_eq!(
            evaluate_shell_command("   ").unwrap(),
            None
        );
        assert_eq!(
            evaluate_shell_command("list --type functional").unwrap(),
            Some(ShellCommandAction::Continue(vec![
                "list".to_string(),
                "--type".to_string(),
                "functional".to_string(),
            ]))
        );
    }

    // trace:DOC-JM-001 | ai:antigravity
    #[test]
    fn faq_loads_and_validates_successfully() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let entries = load_faq(repo_root).expect("FAQ content must load and validate");
        assert!(entries.iter().any(|e| e.id == "faq.canonical_lifecycle"));
    }

    // trace:FR-19,FR-21 | ai:antigravity
    #[test]
    fn launchpad_options_uninitialized_and_initialized() {
        let uninit = launchpad_options(false);
        assert_eq!(
            uninit,
            &[
                LaunchpadOption::Init,
                LaunchpadOption::Help,
                LaunchpadOption::Exit,
            ]
        );
        let uninit_text = launchpad_menu_text(uninit, 0, true);
        assert!(uninit_text.contains("init"));
        assert!(uninit_text.contains("help"));
        assert!(uninit_text.contains("exit"));
        assert!(uninit_text.contains("c to toggle clear [on]"));
        assert!(!uninit_text.contains("agent"));
        assert!(!uninit_text.contains("status"));
        assert!(!uninit_text.contains("shell"));

        let init = launchpad_options(true);
        assert_eq!(
            init,
            &[
                LaunchpadOption::Agent,
                LaunchpadOption::Status,
                LaunchpadOption::Shell,
                LaunchpadOption::Help,
                LaunchpadOption::Exit,
            ]
        );
        let init_text = launchpad_menu_text(init, 0, false);
        assert!(init_text.contains("agent"));
        assert!(init_text.contains("status"));
        assert!(init_text.contains("shell"));
        assert!(init_text.contains("help"));
        assert!(init_text.contains("exit"));
        assert!(init_text.contains("c to toggle clear [off]"));
        assert!(!init_text.contains("init"));
    }

    // trace:FR-19 | ai:antigravity
    #[test]
    fn find_aida_and_git_project_roots() {
        let dir = tmp();
        std::fs::create_dir_all(&dir).unwrap();
        assert_eq!(find_aida_project_root(&dir), None);
        assert_eq!(find_git_project_root(&dir), None);

        // Add git root
        let git_dir = dir.join(".git");
        std::fs::create_dir_all(&git_dir).unwrap();
        std::fs::write(git_dir.join("HEAD"), "ref: refs/heads/main\n").unwrap();
        assert_eq!(find_git_project_root(&dir).as_deref(), Some(dir.as_path()));
        assert_eq!(find_aida_project_root(&dir), None);

        // Nested subfolder in git root
        let sub = dir.join("src/nested");
        std::fs::create_dir_all(&sub).unwrap();
        assert_eq!(find_git_project_root(&sub).as_deref(), Some(dir.as_path()));
        assert_eq!(find_aida_project_root(&sub), None);

        // Add aida root
        let aida_store = dir.join(".aida-store");
        std::fs::create_dir_all(&aida_store).unwrap();
        assert_eq!(find_aida_project_root(&dir).as_deref(), Some(dir.as_path()));
        assert_eq!(find_aida_project_root(&sub).as_deref(), Some(dir.as_path()));

        std::fs::remove_dir_all(&dir).ok();
    }

    // trace:FR-19,FR-21 | ai:antigravity
    #[test]
    fn launchpad_key_navigation_and_actions() {
        let mut selected = 0usize;
        let mut stdin = io::stdin();

        // j down
        let res = handle_launchpad_key(b'j', &mut selected, 2, &mut stdin).unwrap();
        assert_eq!(res, MenuNavAction::Continue);
        assert_eq!(selected, 1);

        // j down to max_idx
        let res = handle_launchpad_key(b'j', &mut selected, 2, &mut stdin).unwrap();
        assert_eq!(res, MenuNavAction::Continue);
        assert_eq!(selected, 2);

        // j clamps at max_idx
        let res = handle_launchpad_key(b'j', &mut selected, 2, &mut stdin).unwrap();
        assert_eq!(res, MenuNavAction::Continue);
        assert_eq!(selected, 2);

        // k up
        let res = handle_launchpad_key(b'k', &mut selected, 2, &mut stdin).unwrap();
        assert_eq!(res, MenuNavAction::Continue);
        assert_eq!(selected, 1);

        // k up to 0
        let res = handle_launchpad_key(b'k', &mut selected, 2, &mut stdin).unwrap();
        assert_eq!(res, MenuNavAction::Continue);
        assert_eq!(selected, 0);

        // k clamps at 0
        let res = handle_launchpad_key(b'k', &mut selected, 2, &mut stdin).unwrap();
        assert_eq!(res, MenuNavAction::Continue);
        assert_eq!(selected, 0);

        // c and C toggle clear
        let res = handle_launchpad_key(b'c', &mut selected, 2, &mut stdin).unwrap();
        assert_eq!(res, MenuNavAction::ToggleClear);
        let res = handle_launchpad_key(b'C', &mut selected, 2, &mut stdin).unwrap();
        assert_eq!(res, MenuNavAction::ToggleClear);

        // Enter selects
        let res = handle_launchpad_key(b'\n', &mut selected, 2, &mut stdin).unwrap();
        assert_eq!(res, MenuNavAction::Select);

        // Ctrl-C cancels
        let res = handle_launchpad_key(3, &mut selected, 2, &mut stdin).unwrap();
        assert_eq!(res, MenuNavAction::Cancel);
    }

    // trace:BUG-15 | ai:antigravity
    #[test]
    fn tutorial_options_visibility_per_status() {
        // NotStarted hides Resume and Repeat
        let opts = tutorial_options(OnboardingStatus::NotStarted);
        assert!(!opts.contains(&TutorialOption::Resume));
        assert!(!opts.contains(&TutorialOption::Repeat));
        assert_eq!(opts[0], TutorialOption::Onboarding);

        // InProgress shows Resume as first option
        let opts = tutorial_options(OnboardingStatus::InProgress { passed: 2 });
        assert!(opts.contains(&TutorialOption::Resume));
        assert!(!opts.contains(&TutorialOption::Repeat));
        assert_eq!(opts[0], TutorialOption::Resume);
        assert_eq!(
            opts[0].label(2, "exit"),
            "resume      — show current onboarding action (Step 2 of 5)"
        );

        // Completed shows Repeat, hides Resume
        let opts = tutorial_options(OnboardingStatus::Completed);
        assert!(!opts.contains(&TutorialOption::Resume));
        assert!(opts.contains(&TutorialOption::Repeat));
        assert_eq!(opts[0], TutorialOption::Repeat);
    }

    // trace:BUG-15 | ai:antigravity
    #[test]
    fn onboarding_status_resolution() {
        let root = tmp();
        let ws = root.join("workspace");
        std::fs::create_dir_all(&ws).unwrap();

        // Fresh repo and workspace -> NotStarted
        let status = onboarding_status(&ws, &root);
        assert_eq!(status, OnboardingStatus::NotStarted);

        // With durable progress showing completed -> Completed
        let progress_file = root.join(".aida-tutor-progress.toml");
        std::fs::write(&progress_file, "onboarding_completed = true\ncompleted = []\n").unwrap();
        let status = onboarding_status(&ws, &root);
        assert_eq!(status, OnboardingStatus::Completed);

        // Reset progress, create agent selection in workspace -> InProgress
        std::fs::write(&progress_file, "onboarding_completed = false\ncompleted = []\n").unwrap();
        save_agent_selection(&ws, &selection(AgentChoice::Claude, &[AgentChoice::Claude])).unwrap();
        let status = onboarding_status(&ws, &root);
        assert!(matches!(status, OnboardingStatus::InProgress { .. }));

        std::fs::remove_dir_all(&root).ok();
    }
}
