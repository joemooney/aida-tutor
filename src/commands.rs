//! Live AIDA command reference. The golden snapshot remains an upstream test fixture.
// trace:STORY-63 | ai:codex
use anyhow::{bail, Context, Result};
use colored::Colorize;
use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use serde::Deserialize;
use std::{
    collections::HashMap,
    io::{self, IsTerminal, Write},
    process::Command,
};

#[derive(Debug, Deserialize)]
struct Entry {
    path: Vec<String>,
    about: String,
    #[serde(default)]
    hidden: bool,
    #[serde(default)]
    flags: Vec<Flag>,
}

#[derive(Debug, Deserialize)]
struct Flag {
    name: String,
}

impl Entry {
    fn name(&self) -> String {
        self.path.join(" ")
    }

    fn matches(&self, query: &str) -> bool {
        let haystack = format!(
            "{} {} {}",
            self.name(),
            self.about,
            self.flags
                .iter()
                .map(|f| format!("--{}", f.name))
                .collect::<Vec<_>>()
                .join(" ")
        )
        .to_lowercase();
        query
            .split_whitespace()
            .all(|word| haystack.contains(&word.to_lowercase()))
    }
}

fn load_catalog() -> Result<Vec<Entry>> {
    let output = Command::new("aida")
        .args(["help", "commands", "--hidden", "--json"])
        .output()
        .context("loading command catalog; install AIDA and put it on PATH")?;
    if !output.status.success() {
        bail!(
            "AIDA command catalog failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let mut entries: Vec<Entry> = serde_json::from_slice(&output.stdout).context(
        "reading AIDA's command catalog; this browser needs `aida help commands --hidden --json`",
    )?;
    // Never accept arbitrary executables from the catalog; only pass command paths to AIDA.
    entries.retain(|e| e.path.first().map(String::as_str) == Some("aida") && e.path.len() > 1);
    // trace:STORY-63 | ai:codex — filtering and help navigation retain this order.
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    entries.insert(
        0,
        Entry {
            path: vec!["aida".into()],
            about: "AIDA root command help".into(),
            hidden: false,
            flags: vec![],
        },
    );
    Ok(entries)
}

fn load_help(entry: &Entry) -> String {
    match Command::new("aida")
        .args(&entry.path[1..])
        .arg("--help")
        .env("NO_COLOR", "1")
        .env("CLICOLOR", "0")
        .env("TERM", "dumb")
        .output()
    {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).into_owned(),
        Ok(out) => format!(
            "Could not load help for {}:\n{}\n{}",
            entry.name(),
            String::from_utf8_lossy(&out.stderr),
            String::from_utf8_lossy(&out.stdout)
        ),
        Err(err) => format!("Could not load help: {err}"),
    }
}

#[derive(Default)]
struct Browser {
    query: String,
    selected: usize,
    detail: bool,
    scroll: usize,
}

impl Browser {
    fn filtered(&self, entries: &[Entry]) -> Vec<usize> {
        entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.matches(&self.query))
            .map(|(i, _)| i)
            .collect()
    }

    fn move_selection(&mut self, forward: bool, len: usize) {
        self.selected = if forward {
            (self.selected + 1).min(len.saturating_sub(1))
        } else {
            self.selected.saturating_sub(1)
        };
        self.scroll = 0;
    }

    fn refilter(&mut self) {
        self.selected = 0;
        self.scroll = 0;
    }
}

struct Screen;
impl Screen {
    fn enter() -> Result<Self> {
        terminal::enable_raw_mode()?;
        let screen = Self;
        execute!(io::stdout(), EnterAlternateScreen, Hide)?;
        Ok(screen)
    }
}
impl Drop for Screen {
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), Show, LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}

/// Bound every physical row so long help lines cannot overwrite the navigation footer.
fn wrap(text: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    text.lines()
        .flat_map(|line| {
            let chars: Vec<char> = line
                .replace('\t', "    ")
                .chars()
                .filter(|c| !c.is_control())
                .collect();
            if chars.is_empty() {
                vec![String::new()]
            } else {
                chars
                    .chunks(width)
                    .map(|chunk| chunk.iter().collect())
                    .collect()
            }
        })
        .collect()
}

fn style_help(line: &str) -> String {
    if line.starts_with("Usage:") || (!line.starts_with(' ') && line.ends_with(':')) {
        return line.cyan().bold().to_string();
    }
    let mut styled = String::new();
    // Preserve whitespace while emphasizing flags, placeholders, and command examples.
    for part in line.split_inclusive(char::is_whitespace) {
        let token = part.trim_end();
        let space = &part[token.len()..];
        if token.starts_with('-') {
            styled.push_str(&token.green().bold().to_string());
        } else if token.starts_with('<') || token.starts_with('[') {
            styled.push_str(&token.yellow().to_string());
        } else if token.starts_with('`') {
            styled.push_str(&token.magenta().to_string());
        } else {
            styled.push_str(token);
        }
        styled.push_str(space);
    }
    styled
}

fn draw(
    entries: &[Entry],
    state: &mut Browser,
    filtered: &[usize],
    cache: &mut HashMap<usize, String>,
) -> Result<usize> {
    let (cols, rows) = terminal::size()?;
    let width = usize::from(cols).saturating_sub(1).max(1);
    let height = usize::from(rows).saturating_sub(5).max(1);
    let mut out = io::stdout();
    execute!(out, crossterm::cursor::MoveTo(0, 0), Clear(ClearType::All))?;
    let position = if filtered.is_empty() {
        0
    } else {
        state.selected + 1
    };
    let title = if state.detail {
        format!(
            "Help {position}/{} · {}",
            filtered.len(),
            filtered
                .get(state.selected)
                .map(|&idx| entries[idx].name())
                .unwrap_or_default()
        )
    } else {
        format!(
            "AIDA Commands · {position}/{} · type to filter",
            filtered.len()
        )
    };
    write!(out, "{}\r\n", wrap(&title, width)[0].cyan().bold())?;
    write!(
        out,
        "{}\r\n\r\n",
        wrap(&format!("Filter: {}", state.query), width)[0].yellow()
    )?;
    let mut max_scroll = 0;
    if state.detail {
        if let Some(&idx) = filtered.get(state.selected) {
            let help = cache.entry(idx).or_insert_with(|| load_help(&entries[idx]));
            let lines = wrap(help, width);
            max_scroll = lines.len().saturating_sub(height);
            state.scroll = state.scroll.min(max_scroll);
            for line in lines.iter().skip(state.scroll).take(height) {
                write!(out, "{}\r\n", style_help(line))?;
            }
        }
    } else if filtered.is_empty() {
        write!(
            out,
            "No matching commands. Backspace edits the filter; Ctrl-U clears it.\r\n"
        )?;
    } else {
        let start = state
            .selected
            .saturating_sub(height / 2)
            .min(filtered.len().saturating_sub(height));
        for (pos, &idx) in filtered.iter().enumerate().skip(start).take(height) {
            let e = &entries[idx];
            let label = format!(
                "{} {}{} — {}",
                if pos == state.selected { "›" } else { " " },
                e.name(),
                if e.hidden { " [hidden]" } else { "" },
                e.about
            );
            let line = &wrap(&label, width)[0];
            if pos == state.selected {
                write!(out, "{}\r\n", line.black().on_cyan().bold())?;
            } else {
                write!(out, "{}\r\n", line)?;
            }
        }
    }
    execute!(out, crossterm::cursor::MoveTo(0, rows.saturating_sub(1)))?;
    let footer = if state.detail {
        "←/→ prev/next match · ↑/↓ scroll · PgUp/PgDn · Esc list · Ctrl-C quit"
    } else {
        "↑/↓ select · Enter help · type filter · Ctrl-U clear · Esc quit"
    };
    write!(out, "{}", wrap(footer, width)[0].dimmed())?;
    out.flush()?;
    Ok(max_scroll)
}

pub fn run(filter: &str) -> Result<()> {
    let entries = load_catalog()?;
    let mut state = Browser {
        query: filter.into(),
        ..Default::default()
    };
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        for idx in state.filtered(&entries) {
            println!("{} — {}", entries[idx].name(), entries[idx].about);
        }
        println!("Run `aida-tutor commands` in a terminal to browse colorized help.");
        return Ok(());
    }
    let _screen = Screen::enter()?;
    let mut cache = HashMap::new();
    loop {
        let filtered = state.filtered(&entries);
        let max_scroll = draw(&entries, &mut state, &filtered, &mut cache)?;
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind == KeyEventKind::Release {
            continue;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('c' | 'd') => break,
                KeyCode::Char('u') if !state.detail => {
                    state.query.clear();
                    state.refilter();
                }
                _ => {}
            }
            continue;
        }
        if state.detail {
            let page = usize::from(terminal::size()?.1).saturating_sub(5).max(1);
            match key.code {
                KeyCode::Esc | KeyCode::Backspace => state.detail = false,
                KeyCode::Left => state.move_selection(false, filtered.len()),
                KeyCode::Right => state.move_selection(true, filtered.len()),
                KeyCode::Up | KeyCode::Char('k') => state.scroll = state.scroll.saturating_sub(1),
                KeyCode::Down | KeyCode::Char('j') => {
                    state.scroll = (state.scroll + 1).min(max_scroll)
                }
                KeyCode::PageUp => state.scroll = state.scroll.saturating_sub(page),
                KeyCode::PageDown | KeyCode::Char(' ') => {
                    state.scroll = (state.scroll + page).min(max_scroll)
                }
                KeyCode::Home => state.scroll = 0,
                KeyCode::End => state.scroll = max_scroll,
                _ => {}
            }
        } else {
            match key.code {
                KeyCode::Esc => break,
                KeyCode::Up => state.move_selection(false, filtered.len()),
                KeyCode::Down => state.move_selection(true, filtered.len()),
                KeyCode::Enter if !filtered.is_empty() => {
                    state.detail = true;
                    state.scroll = 0;
                }
                KeyCode::Backspace => {
                    state.query.pop();
                    state.refilter();
                }
                KeyCode::Char(c) => {
                    state.query.push(c);
                    state.refilter();
                }
                _ => {}
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // trace:STORY-63 | ai:codex
    #[test]
    fn navigation_stays_inside_filtered_results_and_preserves_query() {
        let entries: Vec<Entry> = serde_json::from_str(
            r#"[
            {"path":["aida","queue","add"],"about":"Queue a spec"},
            {"path":["aida","list"],"about":"List specs"},
            {"path":["aida","queue","work"],"about":"Take queued work"}
        ]"#,
        )
        .unwrap();
        let mut browser = Browser {
            query: "QUEUE".into(),
            detail: true,
            scroll: 23,
            ..Default::default()
        };
        let filtered = browser.filtered(&entries);
        assert_eq!(filtered, vec![0, 2]);
        browser.move_selection(true, filtered.len());
        assert_eq!(
            entries[filtered[browser.selected]].name(),
            "aida queue work"
        );
        assert_eq!(browser.scroll, 0);
        browser.move_selection(true, filtered.len());
        assert_eq!(browser.selected, 1);
        browser.move_selection(false, filtered.len());
        assert_eq!(browser.selected, 0);
        assert_eq!(browser.query, "QUEUE");
        browser.query = "no matches".into();
        browser.refilter();
        browser.move_selection(true, 0);
        assert!(browser.filtered(&entries).is_empty());
        assert_eq!(browser.selected, 0);
    }

    #[test]
    fn wrapping_preserves_long_help_and_empty_lines() {
        assert_eq!(
            wrap("Usage: abcdef\n\n  --flag", 6),
            vec!["Usage:", " abcde", "f", "", "  --fl", "ag"]
        );
    }
}
