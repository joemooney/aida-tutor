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

/// The full message of the most recent commit that touched `rel` in
/// `workspace` — `git log -1 --format=%B -- <rel>`. None if the path was
/// never committed or git errors. The commit-pairing exercise uses this
/// to inspect the *learner's* commit (the one that added the artifact),
/// not whatever happens to be at HEAD. trace:STORY-29 | ai:claude
pub fn last_commit_message_for_path(workspace: &Path, rel: &str) -> Option<String> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(workspace)
        .args(["log", "-1", "--format=%B", "--", rel])
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

/// True if any commit in `workspace`'s history has `(<spec>)` in its
/// subject line — the `(FR-1)` linking-commit form AIDA's commit-msg hook
/// validates. Scans `git log --format=%s`; the onboarding slice uses it to
/// confirm the learner's commit names the captured spec. False on any git
/// error or empty repo. trace:STORY-33 | ai:claude
pub fn commit_subject_references(workspace: &Path, spec: &str) -> bool {
    let Ok(out) = std::process::Command::new("git")
        .arg("-C")
        .arg(workspace)
        .args(["log", "--format=%s"])
        .output()
    else {
        return false;
    };
    if !out.status.success() {
        return false;
    }
    let needle = format!("({spec})");
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .any(|line| line.contains(&needle))
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

/// One relationship edge on a requirement. AIDA writes the edge on *both*
/// endpoints: `aida add --parent E` puts a `Child → E` edge on the new
/// requirement and the inverse `Parent → child` edge on `E`; `aida rel add
/// A B --type verifies --bidirectional` puts `Verifies → B` on A and the
/// inverse `VerifiedBy → A` on B. trace:STORY-26 | ai:claude
#[derive(Debug, Clone, Default)]
pub struct Relationship {
    /// `rel_type:` — the role this requirement plays in the edge
    /// (`Parent`, `Child`, `Verifies`, `VerifiedBy`, `References`, …).
    pub rel_type: Option<String>,
    /// `target_id:` — the UUID of the requirement at the other end.
    pub target_id: Option<String>,
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
    /// Edges in the `relationships:` block. trace:STORY-26
    pub relationships: Vec<Relationship>,
}

impl RequirementYaml {
    /// True if this requirement carries an edge of `rel_type` (matched
    /// case-insensitively, so `"child"` matches the stored `Child`)
    /// pointing at `target_uuid`. trace:STORY-26 | ai:claude
    pub fn has_edge(&self, rel_type: &str, target_uuid: &str) -> bool {
        !target_uuid.is_empty()
            && self.relationships.iter().any(|r| {
                r.rel_type
                    .as_deref()
                    .map(|t| t.eq_ignore_ascii_case(rel_type))
                    .unwrap_or(false)
                    && r.target_id.as_deref() == Some(target_uuid)
            })
    }
}

pub fn parse_requirement_yaml(content: &str) -> Option<RequirementYaml> {
    let mut out = RequirementYaml::default();
    // Which YAML list block the current line sits inside. AIDA writes the
    // top-level scalar fields first, then an optional `relationships:`
    // block, then an optional `comments:` block. trace:STORY-26
    enum Block {
        None,
        Relationships,
        Comments,
    }
    let mut block = Block::None;
    for line in content.lines() {
        let trimmed = line.trim_start();
        // An unindented line either opens a block or closes the current
        // one; a `- ` list entry stays part of the block it's under.
        if !line.starts_with(' ') {
            if trimmed.starts_with("relationships:") {
                block = Block::Relationships;
                continue;
            } else if trimmed.starts_with("comments:") {
                block = Block::Comments;
                continue;
            } else if !trimmed.starts_with("- ") {
                block = Block::None;
            }
        }
        match block {
            Block::Relationships => {
                if line.starts_with("- ") {
                    out.relationships.push(Relationship::default());
                }
                // Drop a leading `- ` so the entry's opening line parses
                // like any indented continuation line.
                let field = trimmed.strip_prefix("- ").unwrap_or(trimmed);
                if let Some(rel) = out.relationships.last_mut() {
                    if let Some(v) = field.strip_prefix("rel_type:") {
                        rel.rel_type = Some(strip_yaml_value(v));
                    } else if let Some(v) = field.strip_prefix("target_id:") {
                        rel.target_id = Some(strip_yaml_value(v));
                    }
                }
                continue;
            }
            Block::Comments => {
                // Each top-level "- id: ..." inside `comments:` is one
                // comment. We only count list-of-mapping shapes.
                if line.starts_with("- ") || line.starts_with("  - ") {
                    out.comment_count += 1;
                }
                continue;
            }
            Block::None => {}
        }
        // Top-level scalar fields — reached only outside any list block.
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

/// One linked git worktree, as reported by `git worktree list --porcelain`.
/// `aida session start` adds one such worktree per scoped session.
/// trace:STORY-28 | ai:claude
#[derive(Debug, Clone, Default)]
pub struct GitWorktree {
    /// Absolute path of the worktree directory.
    pub path: String,
    /// Checked-out branch (without the `refs/heads/` prefix), or None for
    /// a detached / bare worktree.
    pub branch: Option<String>,
}

/// Every git worktree of the `workspace` repo. The main worktree is always
/// the first entry (git lists it first), followed by linked worktrees —
/// the `.aida-store` store worktree and any `aida session start` session
/// worktrees. Empty vec if `workspace` isn't a git repo. trace:STORY-28
pub fn git_worktrees(workspace: &Path) -> Vec<GitWorktree> {
    let Ok(out) = std::process::Command::new("git")
        .arg("-C")
        .arg(workspace)
        .args(["worktree", "list", "--porcelain"])
        .output()
    else {
        return Vec::new();
    };
    if !out.status.success() {
        return Vec::new();
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let mut list = Vec::new();
    let mut cur: Option<GitWorktree> = None;
    for line in text.lines() {
        if let Some(p) = line.strip_prefix("worktree ") {
            if let Some(done) = cur.take() {
                list.push(done);
            }
            cur = Some(GitWorktree {
                path: p.trim().to_string(),
                branch: None,
            });
        } else if let Some(b) = line.strip_prefix("branch ") {
            if let Some(c) = cur.as_mut() {
                c.branch = Some(b.trim().trim_start_matches("refs/heads/").to_string());
            }
        }
    }
    if let Some(done) = cur.take() {
        list.push(done);
    }
    list
}

/// The branch checked out in the *main* worktree — the base a session
/// branch forks from. The main worktree is always first in `git worktree
/// list`. None if `workspace` isn't a git repo or HEAD is detached.
/// trace:STORY-28 | ai:claude
pub fn main_worktree_branch(workspace: &Path) -> Option<String> {
    git_worktrees(workspace).into_iter().next().and_then(|w| w.branch)
}

/// Commits on `branch` not reachable from `base` — `git rev-list --count
/// base..branch`. The session-worktree exercise uses this to confirm the
/// learner committed work on the session branch. None on any git error.
/// trace:STORY-28 | ai:claude
pub fn git_commits_ahead(workspace: &Path, base: &str, branch: &str) -> Option<usize> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(workspace)
        .args(["rev-list", "--count", &format!("{base}..{branch}")])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8_lossy(&out.stdout).trim().parse().ok()
}

/// One `aida session start` lease, parsed from `.aida/sessions/<id>.toml`.
/// A scoped session owns a sibling git worktree on its own branch; the
/// lease is the on-disk record of it. `aida session end` deletes the
/// file. Only the fields the session-cluster exercises need are pulled;
/// serde ignores the rest. Fields default to None on a partial/odd file.
/// trace:STORY-28 | ai:claude
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct SessionLease {
    /// `id` — the session id `aida session show` / `end` accept.
    pub id: Option<String>,
    /// `branch` — the worktree's branch, forked at `session start`.
    pub branch: Option<String>,
    /// `worktree_path` — absolute path of the session worktree.
    pub worktree_path: Option<String>,
}

/// Parse every session lease under `.aida/sessions/`. A lease is a plain
/// `<id>.toml`; the sidecar `<id>.activity.toml` / `<id>.manifest.toml`
/// files (whose stems carry a second dot) are skipped. Returns empty vec
/// when no session is active. trace:STORY-28 | ai:claude
pub fn session_leases(workspace: &Path) -> Vec<SessionLease> {
    let dir = workspace.join(".aida").join("sessions");
    let mut out = Vec::new();
    let Ok(read_dir) = std::fs::read_dir(&dir) else {
        return out;
    };
    for entry in read_dir.filter_map(Result::ok) {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("toml") {
            continue;
        }
        // `019e3bbb04a3.toml` → stem `019e3bbb04a3` (a lease).
        // `019e3bbb04a3.activity.toml` → stem `019e3bbb04a3.activity`.
        match path.file_stem().and_then(|s| s.to_str()) {
            Some(stem) if !stem.contains('.') => {}
            _ => continue,
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        if let Ok(lease) = toml::from_str::<SessionLease>(&content) {
            // A real lease names a worktree and a branch; an empty or
            // unrelated toml that happens to parse does not.
            if lease.worktree_path.is_some() && lease.branch.is_some() {
                out.push(lease);
            }
        }
    }
    out
}

/// The session lease whose worktree is checked out on `branch`, if any.
/// The session-cluster exercises pin one branch name and follow that
/// lease across start → work → inspect → end. trace:STORY-28 | ai:claude
pub fn session_lease_for_branch(workspace: &Path, branch: &str) -> Option<SessionLease> {
    session_leases(workspace)
        .into_iter()
        .find(|l| l.branch.as_deref() == Some(branch))
}

/// One `aida` invocation logged by the optional `aida-tutor wrapper`
/// script. Each log line is `<rfc3339-utc-timestamp>\t<args>`; the
/// timestamp column is kept in the file for the learner to read but
/// dropped here — verifiers only ever ask "was this command run?".
/// trace:STORY-22 | ai:claude
#[derive(Debug, Clone)]
pub struct Invocation {
    /// The `aida` arguments, whitespace-split (`["show", "FR-1",
    /// "--comments"]`) — the wrapper's `$*`.
    pub args: Vec<String>,
}

/// Read `.aida-tutor-invocations.log` — the record the optional
/// `aida-tutor wrapper` shim appends to on every `aida` call.
///
/// `None` means the wrapper was never installed (no log file): the
/// learner hasn't opted into invocation tracking, so verifiers fall back
/// to prerequisite-state checks. `Some(_)` — possibly an empty vec —
/// means the wrapper is installed. trace:STORY-22 | ai:claude
pub fn aida_invocations(workspace: &Path) -> Option<Vec<Invocation>> {
    let content =
        std::fs::read_to_string(workspace.join(".aida-tutor-invocations.log")).ok()?;
    let mut out = Vec::new();
    for line in content.lines() {
        let line = line.trim_end();
        if line.is_empty() {
            continue;
        }
        // Drop the leading `<timestamp>\t` column if present.
        let args = line.split_once('\t').map(|(_, rest)| rest).unwrap_or(line);
        out.push(Invocation {
            args: args.split_whitespace().map(str::to_string).collect(),
        });
    }
    Some(out)
}

/// Tri-state: has the learner run `aida <subcommand>` (with every token
/// in `with` also present among the args) through the invocation-logging
/// wrapper?
///
/// - `None` — the wrapper isn't installed; the caller can't tell, and
///   should fall back to prerequisite state.
/// - `Some(true)` — a matching invocation is logged.
/// - `Some(false)` — the wrapper is installed but no matching invocation
///   was logged yet.
///
/// The subcommand is matched against the first non-flag argument, so a
/// global flag before it (`aida --verbose list`) still resolves to
/// `list`. trace:STORY-22 | ai:claude
pub fn invoked(workspace: &Path, subcommand: &str, with: &[&str]) -> Option<bool> {
    let invocations = aida_invocations(workspace)?;
    Some(invocations.iter().any(|inv| {
        let resolved = inv.args.iter().find(|a| !a.starts_with('-'));
        resolved.map(|s| s == subcommand).unwrap_or(false)
            && with
                .iter()
                .all(|want| inv.args.iter().any(|a| a == want))
    }))
}

/// Every learner-authored `*.md` plan under `workspace/docs/plans/`. AIDA
/// archives implementation plans there; `aida plan verify` lints them
/// against the structured template. The `_`-prefixed template
/// (`_TEMPLATE.md`, scaffolded by `aida init`) is excluded: it already
/// carries every required section, so counting it would let the
/// plan-verify exercise pass before the learner writes a real plan.
/// Returns empty vec when the directory is absent. trace:STORY-30 | ai:claude
pub fn plan_files(workspace: &Path) -> Vec<std::path::PathBuf> {
    let dir = workspace.join("docs").join("plans");
    let mut out = Vec::new();
    let Ok(read_dir) = std::fs::read_dir(&dir) else {
        return out;
    };
    for entry in read_dir.filter_map(Result::ok) {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        // Skip `_`-prefixed scaffolding (`_TEMPLATE.md`) — only a
        // learner-authored plan should satisfy the exercise. trace:STORY-30
        let underscore_prefixed = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.starts_with('_'))
            .unwrap_or(false);
        if underscore_prefixed {
            continue;
        }
        out.push(path);
    }
    out.sort();
    out
}

/// Spec ids claimed by more than one requirement in the store, sorted and
/// deduped. `aida db check --collisions` reports exactly this shape; the
/// store-audit exercise's verifier runs the same scan independently.
/// Empty vec for a healthy store. trace:STORY-30 | ai:claude
pub fn duplicate_spec_ids(workspace: &Path) -> Vec<String> {
    use std::collections::HashMap;
    let mut counts: HashMap<String, usize> = HashMap::new();
    for req in all_requirements(workspace) {
        if let Some(id) = req.spec_id {
            *counts.entry(id).or_insert(0) += 1;
        }
    }
    let mut dups: Vec<String> = counts
        .into_iter()
        .filter(|(_, n)| *n > 1)
        .map(|(id, _)| id)
        .collect();
    dups.sort();
    dups
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A throwaway directory under the OS temp dir, unique per call.
    fn temp_workspace() -> std::path::PathBuf {
        let uniq = format!(
            "aida-tutor-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let dir = std::env::temp_dir().join(uniq);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn no_log_means_opt_in_off() {
        let ws = temp_workspace();
        assert!(aida_invocations(&ws).is_none());
        assert!(invoked(&ws, "list", &[]).is_none());
        std::fs::remove_dir_all(&ws).ok();
    }

    #[test]
    fn empty_log_means_opted_in_but_nothing_run() {
        let ws = temp_workspace();
        std::fs::write(ws.join(".aida-tutor-invocations.log"), "").unwrap();
        assert_eq!(invoked(&ws, "list", &[]), Some(false));
        std::fs::remove_dir_all(&ws).ok();
    }

    #[test]
    fn parses_and_matches_logged_invocations() {
        let ws = temp_workspace();
        std::fs::write(
            ws.join(".aida-tutor-invocations.log"),
            "2026-05-18T10:00:00Z\tlist\n\
             2026-05-18T10:01:00Z\tshow FR-1 --comments\n\
             2026-05-18T10:02:00Z\t--verbose search JSON\n",
        )
        .unwrap();
        // bare subcommand match
        assert_eq!(invoked(&ws, "list", &[]), Some(true));
        assert_eq!(invoked(&ws, "show", &[]), Some(true));
        // `--comments` flag must also be present
        assert_eq!(invoked(&ws, "show", &["--comments"]), Some(true));
        // a global flag before the subcommand still resolves
        assert_eq!(invoked(&ws, "search", &[]), Some(true));
        // never ran `status`
        assert_eq!(invoked(&ws, "status", &[]), Some(false));
        // `list` ran, but never with `--comments`
        assert_eq!(invoked(&ws, "list", &["--comments"]), Some(false));
        std::fs::remove_dir_all(&ws).ok();
    }
}
