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

/// Walk `.aida-store/objects/` and return every spec_id whose YAML's
/// `spec_id` field starts with the given prefix (e.g. "VIS", "FR").
/// Returns empty vec if the store is missing or nothing matches.
pub fn requirements_with_prefix(workspace: &Path, prefix: &str) -> Vec<RequirementYaml> {
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
        let Some(req) = parse_requirement_yaml(&content) else {
            continue;
        };
        if req
            .spec_id
            .as_deref()
            .map(|s| s.starts_with(prefix) && (s.len() == prefix.len() || s.as_bytes().get(prefix.len()) == Some(&b'-')))
            .unwrap_or(false)
        {
            out.push(req);
        }
    }
    out
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

/// Best-effort YAML extraction. We only need a handful of fields and
/// don't want to pull in a real YAML parser for this minimal verifier.
/// Each field is matched at the start of a line with `key: value` or
/// `key: "value"`. Fails closed (fields default to None) on anything weird.
#[derive(Debug, Clone, Default)]
pub struct RequirementYaml {
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
