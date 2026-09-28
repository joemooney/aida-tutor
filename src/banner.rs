//! A small Rust port of AIDA's copper shimmer prompt.
//! trace:FR-16,FR-20 | ai:codex,antigravity

use anyhow::{Context, Result};
use clap::ValueEnum;
use std::io::{self, IsTerminal, Write};
use std::os::fd::{AsRawFd, RawFd};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

const SHAPE: [&str; 6] = [
    "     ▄▀▀▄     ",
    "   ▄▀▀▀▀▀▀▄   ",
    "  ▄▀▀▀▀▀▀▀▀▄  ",
    " ▄▀▀▀▀  ▀▀▀▀▄ ",
    "▀▀▀▀      ▀▀▀▀",
    "  α  ι  δ  α  ",
];

const BASE: [(u8, u8, u8); 6] = [
    (236, 214, 191),
    (228, 202, 175),
    (177, 139, 107),
    (130, 88, 58),
    (170, 129, 97),
    (210, 160, 105),
];

const STATIC_LINES: [&str; 6] = [
    "     \x1b[38;2;248;221;190m▄\x1b[0m\x1b[38;2;237;204;174;48;2;237;203;168m▀\x1b[0m\x1b[38;2;213;173;137;48;2;207;161;120m▀\x1b[0m\x1b[38;2;177;131;95m▄\x1b[0m     ",
    "   \x1b[38;2;211;174;146m▄\x1b[0m\x1b[38;2;232;202;171;48;2;255;226;193m▀\x1b[0m\x1b[38;2;244;217;187;48;2;236;205;172m▀\x1b[0m\x1b[38;2;217;179;141;48;2;209;168;129m▀\x1b[0m\x1b[38;2;192;146;105;48;2;185;139;99m▀\x1b[0m\x1b[38;2;177;126;87;48;2;164;117;80m▀\x1b[0m\x1b[38;2;152;111;81;48;2;154;108;73m▀\x1b[0m\x1b[38;2;139;108;80m▄\x1b[0m   ",
    "  \x1b[38;2;177;139;107m▄\x1b[0m\x1b[38;2;236;198;161;48;2;133;100;73m▀\x1b[0m\x1b[38;2;232;205;175;48;2;86;54;34m▀\x1b[0m\x1b[38;2;216;179;144;48;2;93;62;41m▀\x1b[0m\x1b[38;2;211;162;120;48;2;167;128;96m▀\x1b[0m\x1b[38;2;186;135;95;48;2;178;132;92m▀\x1b[0m\x1b[38;2;162;114;78;48;2;169;119;79m▀\x1b[0m\x1b[38;2;148;102;68;48;2;141;98;65m▀\x1b[0m\x1b[38;2;139;99;69;48;2;145;100;66m▀\x1b[0m\x1b[38;2;136;101;75m▄\x1b[0m  ",
    " \x1b[38;2;130;88;58m▄\x1b[0m\x1b[38;2;113;74;49;48;2;138;90;55m▀\x1b[0m\x1b[38;2;89;51;27;48;2;157;109;72m▀\x1b[0m\x1b[38;2;115;73;47;48;2;154;111;79m▀\x1b[0m\x1b[38;2;113;81;60m▀\x1b[0m  \x1b[38;2;165;119;85m▀\x1b[0m\x1b[38;2;157;110;73;48;2;155;112;80m▀\x1b[0m\x1b[38;2;135;91;58;48;2;148;102;67m▀\x1b[0m\x1b[38;2;138;95;65;48;2;139;94;60m▀\x1b[0m\x1b[38;2;135;96;69m▄\x1b[0m ",
    "\x1b[38;2;170;129;97;48;2;193;151;112m▀\x1b[0m\x1b[38;2;182;131;92;48;2;212;167;126m▀\x1b[0m\x1b[38;2;191;140;98;48;2;212;168;130m▀\x1b[0m\x1b[38;2;184;137;100m▀\x1b[0m      \x1b[38;2;151;111;82m▀\x1b[0m\x1b[38;2;140;95;62;48;2;142;104;74m▀\x1b[0m\x1b[38;2;135;91;58;48;2;144;101;69m▀\x1b[0m\x1b[38;2;136;101;76;48;2;138;103;79m▀\x1b[0m",
    "\x1b[38;2;210;160;105m  α  ι  δ  α  \x1b[0m",
];

// The 10-row mask is the metallic arch used by shimmer.py. Each pair of
// rows becomes one terminal row using the Unicode upper-half block.
const ARCH_MASK: [[bool; 14]; 10] = [
    [
        false, false, false, false, false, false, true, true, false, false, false, false, false,
        false,
    ],
    [
        false, false, false, false, false, true, true, true, true, false, false, false, false,
        false,
    ],
    [
        false, false, false, false, true, true, true, true, true, true, false, false, false, false,
    ],
    [
        false, false, false, true, true, true, true, true, true, true, true, false, false, false,
    ],
    [
        false, false, false, true, true, true, true, true, true, true, true, false, false, false,
    ],
    [
        false, false, true, true, true, true, true, true, true, true, true, true, false, false,
    ],
    [
        false, false, true, true, true, true, false, false, true, true, true, true, false, false,
    ],
    [
        false, true, true, true, true, false, false, false, false, true, true, true, true, false,
    ],
    [
        true, true, true, true, false, false, false, false, false, false, true, true, true, true,
    ],
    [
        true, true, true, false, false, false, false, false, false, false, false, true, true, true,
    ],
];

const ARCH_RGB: [[(u8, u8, u8); 14]; 10] = [
    [
        (0, 0, 0),
        (0, 0, 0),
        (0, 0, 0),
        (255, 127, 127),
        (0, 0, 0),
        (236, 214, 191),
        (237, 204, 174),
        (213, 173, 137),
        (184, 149, 117),
        (0, 0, 0),
        (255, 127, 127),
        (0, 0, 0),
        (0, 0, 0),
        (0, 0, 0),
    ],
    [
        (0, 0, 0),
        (0, 0, 0),
        (255, 170, 170),
        (0, 0, 0),
        (228, 202, 175),
        (248, 221, 190),
        (237, 203, 168),
        (207, 161, 120),
        (177, 131, 95),
        (167, 135, 111),
        (127, 0, 0),
        (127, 127, 127),
        (0, 0, 0),
        (0, 0, 0),
    ],
    [
        (0, 0, 0),
        (255, 255, 0),
        (170, 170, 170),
        (170, 127, 127),
        (232, 202, 171),
        (244, 217, 187),
        (217, 179, 141),
        (192, 146, 105),
        (177, 126, 87),
        (152, 111, 81),
        (0, 0, 1),
        (170, 85, 85),
        (0, 0, 0),
        (0, 0, 0),
    ],
    [
        (0, 0, 0),
        (255, 255, 170),
        (0, 0, 0),
        (211, 174, 146),
        (255, 226, 193),
        (236, 205, 172),
        (209, 168, 129),
        (185, 139, 99),
        (164, 117, 80),
        (154, 108, 73),
        (139, 108, 80),
        (0, 0, 0),
        (170, 85, 85),
        (0, 0, 0),
    ],
    [
        (255, 255, 127),
        (0, 0, 0),
        (238, 195, 170),
        (236, 198, 161),
        (232, 205, 175),
        (216, 179, 144),
        (211, 162, 120),
        (186, 135, 95),
        (162, 114, 78),
        (148, 102, 68),
        (139, 99, 69),
        (191, 159, 159),
        (127, 127, 127),
        (255, 0, 0),
    ],
    [
        (127, 127, 63),
        (1, 2, 2),
        (177, 139, 107),
        (133, 100, 73),
        (86, 54, 34),
        (93, 62, 41),
        (167, 128, 96),
        (178, 132, 92),
        (169, 119, 79),
        (141, 98, 65),
        (145, 100, 66),
        (136, 101, 75),
        (0, 0, 0),
        (127, 63, 63),
    ],
    [
        (0, 0, 0),
        (149, 117, 94),
        (113, 74, 49),
        (89, 51, 27),
        (115, 73, 47),
        (113, 81, 60),
        (0, 0, 0),
        (172, 131, 115),
        (165, 119, 85),
        (157, 110, 73),
        (135, 91, 58),
        (138, 95, 65),
        (160, 132, 113),
        (255, 0, 0),
    ],
    [
        (0, 85, 85),
        (130, 88, 58),
        (138, 90, 55),
        (157, 109, 72),
        (154, 111, 79),
        (1, 2, 2),
        (170, 127, 85),
        (0, 0, 0),
        (162, 162, 139),
        (155, 112, 80),
        (148, 102, 67),
        (139, 94, 60),
        (135, 96, 69),
        (0, 0, 0),
    ],
    [
        (170, 129, 97),
        (182, 131, 92),
        (191, 140, 98),
        (184, 137, 100),
        (191, 151, 119),
        (255, 0, 0),
        (127, 127, 127),
        (255, 127, 127),
        (0, 0, 0),
        (0, 1, 2),
        (151, 111, 82),
        (140, 95, 62),
        (135, 91, 58),
        (136, 101, 76),
    ],
    [
        (193, 151, 112),
        (212, 167, 126),
        (212, 168, 130),
        (201, 159, 127),
        (0, 0, 0),
        (255, 127, 127),
        (0, 0, 0),
        (0, 0, 0),
        (127, 127, 127),
        (127, 0, 0),
        (0, 0, 0),
        (142, 104, 74),
        (144, 101, 69),
        (138, 103, 79),
    ],
];

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum BannerLogo {
    /// The compact pyramid-shaped AIDA mark.
    Pyramid,
    /// The metallic arch from shimmer.py.
    Arch,
}

impl BannerLogo {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pyramid => "pyramid",
            Self::Arch => "arch",
        }
    }
}

// trace:FR-20 | ai:antigravity
fn resolve_version_line() -> String {
    if let Ok(out) = Command::new("aida").arg("--version").output() {
        if out.status.success() {
            let text = String::from_utf8_lossy(&out.stdout);
            let first = text.lines().next().unwrap_or("").trim();
            if let Some(rest) = first.strip_prefix("aida ") {
                let ver = rest.split_whitespace().next().unwrap_or("0.15.0");
                return format!("\x1b[1;38;2;225;175;115mAIDA CLI {ver}\x1b[0m");
            }
        }
    }
    format!(
        "\x1b[1;38;2;225;175;115mAIDA Tutor {}\x1b[0m",
        env!("CARGO_PKG_VERSION")
    )
}

// trace:FR-20 | ai:antigravity
fn resolve_user_line(dir: Option<&Path>) -> String {
    if let Ok(author) = std::env::var("AIDA_AUTHOR") {
        let trimmed = author.trim();
        if !trimmed.is_empty() {
            return format!("\x1b[38;2;150;140;130m{trimmed}\x1b[0m");
        }
    }
    let mut cmd = Command::new("git");
    cmd.args(["config", "user.email"]);
    if let Some(d) = dir {
        cmd.current_dir(d);
    }
    if let Ok(out) = cmd.output() {
        if out.status.success() {
            let email = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !email.is_empty() {
                return format!("\x1b[38;2;150;140;130m{email}\x1b[0m");
            }
        }
    }
    let user = std::env::var("USER").unwrap_or_else(|_| "operator".to_string());
    format!("\x1b[38;2;150;140;130m{user}\x1b[0m")
}

// trace:FR-20 | ai:antigravity
fn resolve_role_line(dir: Option<&Path>) -> String {
    let mut cmd = Command::new("aida");
    cmd.arg("statusline");
    if let Some(d) = dir {
        cmd.current_dir(d);
    }
    if let Ok(out) = cmd.output() {
        if out.status.success() {
            let line = String::from_utf8_lossy(&out.stdout).trim().to_string();
            let parts: Vec<&str> = line.split('·').map(str::trim).collect();
            let role_part = parts.iter().find(|p| p.starts_with("role:"));
            let spec_part = parts.iter().find(|p| p.starts_with('@'));
            if let Some(role) = role_part {
                if let Some(spec) = spec_part {
                    return format!("\x1b[38;2;150;140;130m{role} · {spec}\x1b[0m");
                } else {
                    return format!("\x1b[38;2;150;140;130m{role}\x1b[0m");
                }
            }
        }
    }
    let role = std::env::var("AIDA_SESSION_ROLE").unwrap_or_else(|_| "advisor".to_string());
    format!("\x1b[38;2;150;140;130mrole:{role}\x1b[0m")
}

// trace:FR-20 | ai:antigravity
fn shorten_path(path: &Path) -> String {
    let path_str = path.to_string_lossy();
    if let Some(home) = std::env::var_os("HOME").and_then(|h| h.into_string().ok()) {
        if path_str == home {
            return "~".to_string();
        }
        let prefix = format!("{home}/");
        if let Some(rest) = path_str.strip_prefix(&prefix) {
            return format!("~/{rest}");
        }
    }
    path_str.to_string()
}

// trace:FR-20 | ai:antigravity
fn extract_project_name(root: &Path) -> String {
    let metadata_path = root.join(".aida-store/metadata.yaml");
    if let Ok(content) = std::fs::read_to_string(&metadata_path) {
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(name) = trimmed.strip_prefix("name:") {
                let name = name.trim().trim_matches('\'').trim_matches('"');
                if !name.is_empty() {
                    return name.to_string();
                }
            }
        }
    }
    root.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("aida")
        .to_string()
}

// trace:FR-20 | ai:antigravity
fn resolve_git_branch(dir: &Path) -> Option<String> {
    if let Ok(out) = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(dir)
        .output()
    {
        if out.status.success() {
            let b = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !b.is_empty() {
                return Some(b);
            }
        }
    }
    let head_file = dir.join(".git/HEAD");
    if let Ok(content) = std::fs::read_to_string(head_file) {
        let trimmed = content.trim();
        if let Some(ref_path) = trimmed.strip_prefix("ref: refs/heads/") {
            return Some(ref_path.to_string());
        }
    }
    None
}

// trace:FR-20 | ai:antigravity
fn count_specs(root: &Path) -> usize {
    let objects_dir = root.join(".aida-store/objects");
    if !objects_dir.is_dir() {
        return 0;
    }
    fn count_in_dir(dir: &Path, depth: usize) -> usize {
        if depth > 4 {
            return 0;
        }
        let mut count = 0;
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    count += count_in_dir(&path, depth + 1);
                } else if path.is_file()
                    && path.extension().map_or(false, |ext| ext == "yaml" || ext == "json")
                {
                    count += 1;
                }
            }
        }
        count
    }
    count_in_dir(&objects_dir, 0)
}

// trace:FR-20 | ai:antigravity
pub(crate) fn resolve_project_and_store(dir: &Path) -> (String, String) {
    let short_path = shorten_path(dir);

    let aida_root = crate::onboarding::find_aida_project_root(dir);
    let Some(root) = aida_root else {
        let p_line = "no AIDA project (use 'init' to create one)".to_string();
        let s_line = format!("{short_path} · uninitialized");
        return (
            format!("\x1b[38;2;150;140;130m{p_line}\x1b[0m"),
            format!("\x1b[38;2;110;105;100m{s_line}\x1b[0m"),
        );
    };

    let is_root = match (dir.canonicalize(), root.canonicalize()) {
        (Ok(d), Ok(r)) => d == r,
        _ => dir == root,
    };

    let project_name = extract_project_name(&root);
    let branch = resolve_git_branch(&root).or_else(|| resolve_git_branch(dir));
    let branch_str = branch.map(|b| format!(" ({b})")).unwrap_or_default();
    let spec_count = count_specs(&root);

    let p_line = if is_root {
        format!("project: {project_name}{branch_str}")
    } else {
        format!("parent aida-store: {project_name}{branch_str}")
    };

    let s_line = if spec_count == 1 {
        format!("{short_path} · 1 spec")
    } else {
        format!("{short_path} · {spec_count} specs")
    };

    (
        format!("\x1b[38;2;150;140;130m{p_line}\x1b[0m"),
        format!("\x1b[38;2;110;105;100m{s_line}\x1b[0m"),
    )
}

// trace:FR-20 | ai:antigravity
fn get_info_lines(dir: Option<&Path>) -> [String; 5] {
    let current_dir = dir
        .map(|p| p.to_path_buf())
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));

    let version_line = resolve_version_line();
    let user_line = resolve_user_line(Some(&current_dir));
    let role_line = resolve_role_line(Some(&current_dir));
    let (project_line, specs_line) = resolve_project_and_store(&current_dir);

    [version_line, user_line, role_line, project_line, specs_line]
}

/// Render the configured banner until the first byte arrives, then return that
/// byte to the caller so menus and prompts do not lose the user's first
/// keystroke.
// trace:FR-16,FR-20 | ai:codex,antigravity
pub fn show_and_read(under: &str, dir: Option<&Path>) -> Result<Option<u8>> {
    match banner_mode() {
        BannerMode::Static => show_static_and_read(under, dir),
        BannerMode::Shimmer => show_shimmer_and_read(under, dir),
        BannerMode::Off => Ok(None),
    }
}

/// Redraw a menu after its selection changes. Repainting the whole screen
/// avoids relying on terminal row counts when ANSI-colored banner lines wrap.
// trace:FR-16,FR-20 | ai:codex,antigravity
pub fn redraw_menu(under: &str, dir: Option<&Path>) -> Result<()> {
    let mut stdout = io::stdout();
    write!(stdout, "\x1b[2J\x1b[H")?;
    write_banner(&mut stdout, dir)?;
    write!(stdout, "{under}\n")?;
    stdout.flush()?;
    Ok(())
}

#[derive(Clone, Copy)]
enum BannerMode {
    Static,
    Shimmer,
    Off,
}

fn banner_mode() -> BannerMode {
    let configured = std::env::var("AIDA_TUTOR_BANNER")
        .ok()
        .or_else(|| config_value("banner"));

    match configured
        .map(|value| value.to_ascii_lowercase())
        .as_deref()
    {
        Some("static") => BannerMode::Static,
        Some("off" | "none" | "false") => BannerMode::Off,
        Some("shimmer") | None => BannerMode::Shimmer,
        Some(_) => BannerMode::Shimmer,
    }
}

fn logo_mode() -> BannerLogo {
    let configured = std::env::var("AIDA_TUTOR_LOGO")
        .ok()
        .or_else(|| config_value("logo"));
    match configured
        .as_deref()
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("arch") => BannerLogo::Arch,
        _ => BannerLogo::Pyramid,
    }
}

fn config_value(key: &str) -> Option<String> {
    let path = std::env::var_os("AIDA_TUTOR_CONFIG")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::env::current_dir()
                .unwrap_or_default()
                .join(".aida-tutor.toml")
        });
    let contents = std::fs::read_to_string(path).ok()?;
    let document: toml::Value = contents.parse().ok()?;
    document.get(key)?.as_str().map(str::to_owned)
}

fn show_static_and_read(under: &str, dir: Option<&Path>) -> Result<Option<u8>> {
    if !io::stdin().is_terminal() {
        return Ok(None);
    }
    let stdin = io::stdin();
    let fd = stdin.as_raw_fd();
    let _terminal = TerminalGuard::enter()?;
    let mut stdout = io::stdout();
    write!(stdout, "\x1b[?25l")?;
    write_banner(&mut stdout, dir)?;
    write!(stdout, "{under}")?;
    stdout.flush()?;
    let byte = read_first_byte(fd)?;
    write!(stdout, "\x1b[?25h\x1b[0m")?;
    stdout.flush()?;
    Ok(byte)
}

fn write_banner(stdout: &mut impl Write, dir: Option<&Path>) -> Result<()> {
    let info_lines = get_info_lines(dir);
    match banner_mode() {
        BannerMode::Static => {
            for (index, line) in STATIC_LINES.iter().enumerate() {
                if index < info_lines.len() {
                    writeln!(stdout, "{line}    {}", info_lines[index])?;
                } else {
                    writeln!(stdout, "{line}")?;
                }
            }
            writeln!(stdout)?;
        }
        BannerMode::Shimmer => {
            for line in render_frame(12.0, true, 1.0, &info_lines) {
                writeln!(stdout, "{line}")?;
            }
            writeln!(stdout)?;
        }
        BannerMode::Off => {}
    }
    Ok(())
}

fn show_shimmer_and_read(under: &str, dir: Option<&Path>) -> Result<Option<u8>> {
    if !io::stdin().is_terminal() {
        return Ok(None);
    }

    let info_lines = get_info_lines(dir);
    let stdin = io::stdin();
    let fd = stdin.as_raw_fd();
    let _terminal = TerminalGuard::enter()?;
    let mut stdout = io::stdout();
    write!(stdout, "\x1b[?25l")?;
    stdout.flush()?;

    let under_lines = under.lines().count().max(1);
    let start = Instant::now();
    let mut first = true;
    loop {
        let elapsed = start.elapsed().as_secs_f64();
        let cycle = (elapsed % 2.4) / 2.4;
        let active = cycle < 0.65;
        let sweep = if active {
            -4.0 + (cycle / 0.65) * 26.0
        } else {
            999.0
        };
        let ambient = 1.0 + 0.03 * (cycle * std::f64::consts::TAU).sin();
        let frame = render_frame(sweep, active, ambient, &info_lines);
        if !first {
            write!(stdout, "\x1b[{}A\r", under_lines + 6)?;
        }
        for line in &frame {
            write!(stdout, "{}\x1b[K\n", line)?;
        }
        write!(stdout, "\x1b[K\n{}", under)?;
        stdout.flush()?;
        first = false;

        let mut pollfd = libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        };
        let ready = unsafe { libc::poll(&mut pollfd, 1, 33) };
        if ready > 0 && pollfd.revents & libc::POLLIN != 0 {
            let mut byte = [0u8; 1];
            let count =
                unsafe { libc::read(fd, byte.as_mut_ptr() as *mut libc::c_void, byte.len()) };
            write!(stdout, "\x1b[?25h\x1b[0m")?;
            stdout.flush()?;
            return Ok((count == 1).then_some(byte[0]));
        }
    }
}

fn render_frame(sweep: f64, active: bool, ambient: f64, info_lines: &[String; 5]) -> Vec<String> {
    let mut lines = match logo_mode() {
        BannerLogo::Pyramid => render_pyramid(sweep, active, ambient),
        BannerLogo::Arch => render_arch(sweep, active, ambient),
    };
    for (index, info) in info_lines.iter().enumerate() {
        lines[index].push_str("    ");
        lines[index].push_str(info);
    }
    lines
}

fn render_pyramid(sweep: f64, active: bool, ambient: f64) -> Vec<String> {
    SHAPE
        .iter()
        .enumerate()
        .map(|(row, source)| {
            let (r0, g0, b0) = BASE[row];
            let mut line = String::new();
            for (column, ch) in source.chars().enumerate() {
                if ch.is_whitespace() {
                    line.push(ch);
                    continue;
                }
                let u = column as f64 + row as f64 * 0.75;
                let distance = u - sweep;
                let glint = if active {
                    (-(distance * distance) / (2.0 * 1.6_f64.powi(2))).exp()
                } else {
                    0.0
                };
                let r = shade(r0, ambient, glint, 255, 0.95, 40.0);
                let g = shade(g0, ambient, glint, 250, 0.90, 30.0);
                let b = shade(b0, ambient, glint, 230, 0.80, 20.0);
                line.push_str(&format!("\x1b[38;2;{r};{g};{b}m{ch}\x1b[0m"));
            }
            line
        })
        .collect()
}

fn render_arch(sweep: f64, active: bool, ambient: f64) -> Vec<String> {
    let mut lines = Vec::with_capacity(6);
    for row in 0..5 {
        let top = row * 2;
        let bottom = top + 1;
        let mut line = String::new();
        for column in 0..14 {
            let top_on = ARCH_MASK[top][column];
            let bottom_on = ARCH_MASK[bottom][column];
            if !top_on && !bottom_on {
                line.push(' ');
                continue;
            }
            let top_color = arch_shade(top, column, sweep, active, ambient);
            let bottom_color = arch_shade(bottom, column, sweep, active, ambient);
            match (top_on, bottom_on) {
                (true, true) => line.push_str(&format!(
                    "\x1b[38;2;{};{};{};48;2;{};{};{}m▀\x1b[0m",
                    top_color.0,
                    top_color.1,
                    top_color.2,
                    bottom_color.0,
                    bottom_color.1,
                    bottom_color.2
                )),
                (true, false) => line.push_str(&format!(
                    "\x1b[38;2;{};{};{}m▀\x1b[0m",
                    top_color.0, top_color.1, top_color.2
                )),
                (false, true) => line.push_str(&format!(
                    "\x1b[38;2;{};{};{}m▄\x1b[0m",
                    bottom_color.0, bottom_color.1, bottom_color.2
                )),
                _ => unreachable!(),
            }
        }
        lines.push(line);
    }
    let mut greek = String::from("  ");
    for (index, ch) in ['α', 'ι', 'δ', 'α'].into_iter().enumerate() {
        let u = 6.0 + [1.0, 4.0, 7.0, 10.0][index] * 0.9;
        let glint = if active {
            (-(u - sweep).powi(2) / (2.0 * 1.5_f64.powi(2))).exp()
        } else {
            0.0
        };
        let r = (210.0 + glint * 75.0).clamp(0.0, 255.0) as u8;
        let g = (160.0 + glint * 110.0).clamp(0.0, 255.0) as u8;
        let b = (105.0 + glint * 140.0).clamp(0.0, 255.0) as u8;
        greek.push_str(&format!("\x1b[38;2;{r};{g};{b}m{ch}\x1b[0m  "));
    }
    lines.push(greek);
    lines
}

fn arch_shade(row: usize, column: usize, sweep: f64, active: bool, ambient: f64) -> (u8, u8, u8) {
    let (r0, g0, b0) = ARCH_RGB[row][column];
    let u = column as f64 + row as f64 * 0.75;
    let glint = if active {
        (-(u - sweep).powi(2) / (2.0 * 1.6_f64.powi(2))).exp()
    } else {
        0.0
    };
    (
        shade(r0, ambient, glint, 255, 0.95, 40.0),
        shade(g0, ambient, glint, 250, 0.90, 30.0),
        shade(b0, ambient, glint, 230, 0.80, 20.0),
    )
}

fn read_first_byte(fd: RawFd) -> Result<Option<u8>> {
    loop {
        let mut pollfd = libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        };
        let ready = unsafe { libc::poll(&mut pollfd, 1, -1) };
        if ready < 0 {
            anyhow::bail!("waiting for banner input failed");
        }
        if ready > 0 && pollfd.revents & libc::POLLIN != 0 {
            let mut byte = [0u8; 1];
            let count =
                unsafe { libc::read(fd, byte.as_mut_ptr() as *mut libc::c_void, byte.len()) };
            return Ok((count == 1).then_some(byte[0]));
        }
    }
}

fn shade(base: u8, ambient: f64, glint: f64, target: u8, weight: f64, extra: f64) -> u8 {
    (base as f64 * ambient + glint * (target as f64 - base as f64) * weight + glint * extra)
        .clamp(0.0, 255.0) as u8
}

struct TerminalGuard {
    saved: String,
}

impl TerminalGuard {
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
            .context("enabling shimmer input")?;
        if !status.success() {
            anyhow::bail!("could not enable shimmer input");
        }
        Ok(Self { saved })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let result = if self.saved.is_empty() {
            Command::new("stty").arg("sane").status()
        } else {
            Command::new("stty").arg(&self.saved).status()
        };
        let _ = result;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // trace:FR-20 | ai:antigravity
    #[test]
    fn test_resolve_project_and_store_states() {
        let temp_base = std::env::temp_dir().join(format!("aida_test_banner_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_base);
        let project_dir = temp_base.join("my-test-proj");
        let sub_dir = project_dir.join("sub/workspace");
        let non_project_dir = temp_base.join("other");

        fs::create_dir_all(&sub_dir).unwrap();
        fs::create_dir_all(&non_project_dir).unwrap();

        // 1. Before initializing AIDA in project_dir
        let (p1, s1) = resolve_project_and_store(&project_dir);
        assert!(p1.contains("no AIDA project"), "Expected no AIDA project, got: {p1}");
        assert!(s1.contains("uninitialized"), "Expected uninitialized, got: {s1}");

        // 2. Initialize .aida-store in project_dir
        let store_dir = project_dir.join(".aida-store");
        fs::create_dir_all(store_dir.join("objects/FR")).unwrap();
        fs::write(
            store_dir.join("metadata.yaml"),
            "name: my-test-proj\ntitle: Test Project\n",
        )
        .unwrap();
        fs::write(store_dir.join("objects/FR/FR-1.yaml"), "title: Req 1\n").unwrap();

        // Check project root
        let (p2, s2) = resolve_project_and_store(&project_dir);
        assert!(p2.contains("project: my-test-proj"), "Expected project: my-test-proj, got: {p2}");
        assert!(s2.contains("1 spec"), "Expected 1 spec, got: {s2}");

        // Check subdirectory using parent store
        let (p3, s3) = resolve_project_and_store(&sub_dir);
        assert!(
            p3.contains("parent aida-store: my-test-proj"),
            "Expected parent aida-store: my-test-proj, got: {p3}"
        );
        assert!(s3.contains("1 spec"), "Expected 1 spec, got: {s3}");

        // Check non-project dir
        let (p4, s4) = resolve_project_and_store(&non_project_dir);
        assert!(p4.contains("no AIDA project"), "Expected no AIDA project, got: {p4}");
        assert!(s4.contains("uninitialized"), "Expected uninitialized, got: {s4}");

        let _ = fs::remove_dir_all(&temp_base);
    }

    // trace:FR-20 | ai:antigravity
    #[test]
    fn test_shorten_path() {
        if let Some(home) = std::env::var_os("HOME").and_then(|h| h.into_string().ok()) {
            let home_path = PathBuf::from(&home);
            assert_eq!(shorten_path(&home_path), "~");
            let sub = home_path.join("some/project");
            assert_eq!(shorten_path(&sub), "~/some/project");
        }
        let tmp = Path::new("/tmp/test");
        assert_eq!(shorten_path(tmp), "/tmp/test");
    }
}
