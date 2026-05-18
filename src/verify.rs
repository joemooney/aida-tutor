//! Helpers for inspecting on-disk state. trace:PRIN-1 | ai:claude
//!
//! Verifiers compose these primitives — none mutate state.

use std::path::Path;

/// True if `workspace/<rel>` exists.
pub fn path_exists(workspace: &Path, rel: &str) -> bool {
    workspace.join(rel).exists()
}

/// True if the workspace looks like an AIDA-initialized repo:
/// `.aida/config.toml` AND `.aida-store/metadata.yaml` both exist.
pub fn is_aida_initialized(workspace: &Path) -> bool {
    path_exists(workspace, ".aida/config.toml")
        && path_exists(workspace, ".aida-store/metadata.yaml")
}

/// Walk `.aida-store/objects/` and return every requirement whose YAML
/// parses. Returns empty vec if the store is missing. trace:STORY-27
pub fn all_requirements(workspace: &Path) -> Vec<RequirementYaml> {
    let objects = workspace.join(".aida-store").join("objects");
    if !objects.exists() {
        return Vec::new();
    }
    let mut out = Vec::new();
    for entry in walkdir::WalkDir::new(&objects)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("yaml") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        if let Some(req) = parse_requirement_yaml(&content) {
            out.push(req);
        }
    }
    out
}

/// Requirements whose `spec_id` starts with the given prefix (e.g. "VIS",
/// "FR"), matched on a type-prefix boundary so "FR" doesn't also catch a
/// hypothetical "FROZEN-1". Returns empty vec if nothing matches.
pub fn requirements_with_prefix(workspace: &Path, prefix: &str) -> Vec<RequirementYaml> {
    all_requirements(workspace)
        .into_iter()
        .filter(|req| {
            req.spec_id
                .as_deref()
                .map(|s| {
                    s.starts_with(prefix)
                        && (s.len() == prefix.len()
                            || s.as_bytes().get(prefix.len()) == Some(&b'-'))
                })
                .unwrap_or(false)
        })
        .collect()
}

/// First requirement whose `title` contains `needle` (case-insensitive).
/// The queue-cluster exercises (22-24) follow one task by title across
/// capture → pickup → done. trace:STORY-27 | ai:claude
pub fn requirement_by_title(workspace: &Path, needle: &str) -> Option<RequirementYaml> {
    let needle = needle.to_lowercase();
    all_requirements(workspace).into_iter().find(|r| {
        r.title
            .as_deref()
            .map(|t| t.to_lowercase().contains(&needle))
            .unwrap_or(false)
    })
}

/// Run `git log -1 --format=%s%n%n%b` in workspace and return the message,
/// or None on any error / empty repo.
pub fn last_commit_message(workspace: &Path) -> Option<String> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(workspace)
        .args(["log", "-1", "--format=%B"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&output.stdout).to_string();
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// True if `branch` exists as a local git branch in `workspace`. Used by
/// the distributed-store exercises to confirm the orphan `aida-store`
/// branch is present. trace:STORY-25 | ai:claude
pub fn git_branch_exists(workspace: &Path, branch: &str) -> bool {
    std::process::Command::new("git")
        .arg("-C")
        .arg(workspace)
        .args(["rev-parse", "--verify", "--quiet"])
        .arg(format!("refs/heads/{branch}"))
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Count of commits on `branch` in `workspace`, or None if the branch is
/// missing / git errors. The orphan `aida-store` branch carries one
/// commit per store mutation. trace:STORY-25 | ai:claude
pub fn git_commit_count(workspace: &Path, branch: &str) -> Option<usize> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(workspace)
        .args(["rev-list", "--count", branch])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8_lossy(&out.stdout).trim().parse().ok()
}

/// True if `workspace/<rel>` is a *linked git worktree* — its `.git` entry
/// is a file (a `gitdir:` pointer), not a directory. AIDA's distributed
/// store lives in the `.aida-store/` linked worktree. trace:STORY-25
pub fn is_linked_worktree(workspace: &Path, rel: &str) -> bool {
    workspace.join(rel).join(".git").is_file()
}

/// True if `workspace/.aida/cache.db` exists and begins with the SQLite
/// file magic (`SQLite format 3\0`). A bare presence check would pass on a
/// garbage-filled file; reading the header confirms the cache is a real,
/// openable database — the state `aida cache rebuild` restores.
/// trace:STORY-25 | ai:claude
pub fn cache_db_is_valid_sqlite(workspace: &Path) -> bool {
    use std::io::Read;
    let path = workspace.join(".aida").join("cache.db");
    let Ok(mut f) = std::fs::File::open(&path) else {
        return false;
    };
    let mut head = [0u8; 16];
    if f.read_exact(&mut head).is_err() {
        return false;
    }
    &head == b"SQLite format 3\0"
}

/// Best-effort YAML extraction. We only need a handful of fields and
/// don't want to pull in a real YAML parser for this minimal verifier.
/// Each field is matched at the start of a line with `key: value` or
/// `key: "value"`. Fails closed (fields default to None) on anything weird.
#[derive(Debug, Clone, Default)]
pub struct RequirementYaml {
    /// Top-level `id:` — the UUID queue entries reference. trace:STORY-27
    pub uuid: Option<String>,
    pub spec_id: Option<String>,
    pub title: Option<String>,
    pub status: Option<String>,
    pub req_type: Option<String>,
    pub comment_count: usize,
}

pub fn parse_requirement_yaml(content: &str) -> Option<RequirementYaml> {
    let mut out = RequirementYaml::default();
    let mut in_comments_block = false;
    for line in content.lines() {
        let trimmed = line.trim_start();
        if let Some(v) = trimmed.strip_prefix("spec_id:") {
            out.spec_id = Some(strip_yaml_value(v));
        } else if let Some(v) = trimmed.strip_prefix("id:") {
            // The requirement's UUID. First match wins — the real `id:` is
            // line 1, well before any description text. trace:STORY-27
            if out.uuid.is_none() {
                out.uuid = Some(strip_yaml_value(v));
            }
        } else if let Some(v) = trimmed.strip_prefix("title:") {
            out.title = Some(strip_yaml_value(v));
        } else if let Some(v) = trimmed.strip_prefix("status:") {
            out.status = Some(strip_yaml_value(v));
        } else if let Some(v) = trimmed.strip_prefix("req_type:") {
            out.req_type = Some(strip_yaml_value(v));
        } else if trimmed.starts_with("comments:") {
            in_comments_block = true;
            continue;
        } else if in_comments_block && trimmed.starts_with("- ") {
            // Each top-level "- id: ..." inside `comments:` is one comment.
            // We only count list-of-mapping shapes; nested structures are
            // accounted for by the enclosing pattern.
            if line.starts_with("  - ") || line.starts_with("- ") {
                out.comment_count += 1;
            }
        } else if in_comments_block && !trimmed.is_empty() && !line.starts_with(' ') {
            // Left the comments block.
            in_comments_block = false;
        }
    }
    if out.spec_id.is_none() && out.title.is_none() {
        return None;
    }
    Some(out)
}

fn strip_yaml_value(raw: &str) -> String {
    raw.trim()
        .trim_end_matches('"')
        .trim_start_matches('"')
        .trim_end_matches('\'')
        .trim_start_matches('\'')
        .to_string()
}

/// Recursively grep `workspace` for any line containing `trace:<PREFIX>-`,
/// returning the matched spec_ids (without the `trace:` prefix).
pub fn trace_comments_in_workspace(workspace: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let re = regex::Regex::new(r"trace:([A-Z]+(?:-[0-9]+){1,2})").unwrap();
    for entry in walkdir::WalkDir::new(workspace)
        .into_iter()
        .filter_entry(|e| {
            // skip the AIDA store + git internals + target/
            let n = e.file_name().to_string_lossy();
            n != ".aida-store" && n != ".git" && n != "target" && n != ".aida"
        })
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        for cap in re.captures_iter(&content) {
            if let Some(m) = cap.get(1) {
                out.push(m.as_str().to_string());
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

/// One entry in a per-project work queue. The queue lives at
/// `.aida-store/registry/queues/<user>.yaml` as a YAML list; each entry
/// is a `- user_id:` block. We only need the requirement it points at
/// and the role it's routed to. trace:STORY-27 | ai:claude
#[derive(Debug, Clone, Default)]
pub struct QueueEntry {
    /// `requirement_id` — the UUID of the queued requirement.
    pub requirement_id: Option<String>,
    /// `for_role` — the role this item was routed to via `--for`, if any.
    pub for_role: Option<String>,
}

/// Parse every queue file under `.aida-store/registry/queues/`. Returns
/// empty vec when the store has no queues dir or every queue is empty
/// (an empty queue file is the literal `[]`). trace:STORY-27 | ai:claude
pub fn queue_entries(workspace: &Path) -> Vec<QueueEntry> {
    let dir = workspace
        .join(".aida-store")
        .join("registry")
        .join("queues");
    let mut out = Vec::new();
    let Ok(read_dir) = std::fs::read_dir(&dir) else {
        return out;
    };
    for entry in read_dir.filter_map(Result::ok) {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("yaml") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let mut cur: Option<QueueEntry> = None;
        for line in content.lines() {
            let trimmed = line.trim_start();
            // A `- ` at the start of a (trimmed) line opens a new entry.
            if trimmed.starts_with("- ") {
                if let Some(done) = cur.take() {
                    out.push(done);
                }
                cur = Some(QueueEntry::default());
            }
            // Drop a leading `- ` so the first field on the opening line
            // parses like any other.
            let field = trimmed.strip_prefix("- ").unwrap_or(trimmed);
            if let Some(e) = cur.as_mut() {
                if let Some(v) = field.strip_prefix("requirement_id:") {
                    e.requirement_id = Some(strip_yaml_value(v));
                } else if let Some(v) = field.strip_prefix("for_role:") {
                    e.for_role = Some(strip_yaml_value(v));
                }
            }
        }
        if let Some(done) = cur.take() {
            out.push(done);
        }
    }
    out
}
