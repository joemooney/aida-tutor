//! Exercise 35 — `aida mcp-serve`: talk to AIDA over the Model Context
//! Protocol, the channel an AI agent uses to read and write the store.
//! Cluster 6 (plans + store maintenance + MCP) — and the final exercise.
//! trace:STORY-30 | ai:claude

use anyhow::Context;
use crate::exercise::{Exercise, VerifyResult};
use crate::verify::is_aida_initialized;
use std::path::Path;

pub struct E;

/// Where the content tells the learner to capture the MCP response.
/// `aida mcp-serve` writes to stdout; the exercise redirects it to a file
/// the verifier can read back. trace:STORY-30 | ai:claude
const PROBE: &str = "mcp-probe.json";

/// One MCP `tools/call` request — invoke AIDA's `list_requirements` tool.
const REQUEST: &str =
    r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"list_requirements","arguments":{}}}"#;

impl Exercise for E {
    fn id(&self) -> u32 { 35 }
    fn slug(&self) -> &'static str { "mcp-serve" }
    fn title(&self) -> &'static str { "aida mcp-serve — AIDA over the Model Context Protocol" }
    fn hint(&self) -> &'static str {
        "Back in exercise 01, `aida init` wrote a `.mcp.json` file at the repo root — it tells \
         Claude Code to launch `aida mcp-serve`, the bridge that lets an AI agent read and write \
         requirements without shelling out. `aida mcp-serve` speaks JSON-RPC 2.0 over \
         stdin/stdout. You normally never run it by hand, but doing it once demystifies it: pipe \
         a `tools/call` request in and capture the reply. From `workspace/`, run \
         `echo '<request>' | aida mcp-serve > mcp-probe.json`, then open `mcp-probe.json` to \
         read AIDA's response (`aida-tutor show` prints the exact request)."
    }
    fn verify(&self, workspace: &Path) -> VerifyResult {
        if !is_aida_initialized(workspace) {
            return VerifyResult::Pending("complete exercise 01 first".into());
        }
        let Ok(content) = std::fs::read_to_string(workspace.join(PROBE)) else {
            return VerifyResult::Pending(format!(
                "no `{PROBE}` yet — from `workspace/`, pipe an MCP request into `aida mcp-serve` \
                 and redirect the reply: `echo '...' | aida mcp-serve > {PROBE}` (run \
                 `aida-tutor show` for the exact request)."
            ));
        };
        // A successful `tools/call` reply is a JSON-RPC 2.0 result object
        // carrying a `content` array. Check the three markers so a stray
        // or error-only file can't pass. No JSON parser — substring checks
        // match the tutor's best-effort verifier style (see verify.rs).
        let ok = content.contains("\"jsonrpc\"")
            && content.contains("\"result\"")
            && content.contains("\"content\"");
        if !ok {
            return VerifyResult::Fail(format!(
                "`{PROBE}` doesn't look like an MCP `tools/call` reply — it should be a \
                 JSON-RPC response with a `result` holding a `content` array. Regenerate it: \
                 `echo '...' | aida mcp-serve > {PROBE}`."
            ));
        }
        VerifyResult::Pass
    }
    fn demo(&self, workspace: &Path) -> anyhow::Result<()> {
        // Drive one MCP request by hand: spawn the server, write a single
        // JSON-RPC line, close stdin (EOF → the server answers and exits),
        // and capture stdout to the probe file.
        use std::io::Write;
        use std::process::{Command, Stdio};
        let mut child = Command::new("aida")
            .current_dir(workspace)
            .arg("mcp-serve")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("spawning `aida mcp-serve`")?;
        {
            let mut stdin = child
                .stdin
                .take()
                .context("`aida mcp-serve` stdin unavailable")?;
            writeln!(stdin, "{REQUEST}")?;
        } // stdin handle dropped here → EOF → mcp-serve answers and exits
        let out = child
            .wait_with_output()
            .context("waiting on `aida mcp-serve`")?;
        if !out.status.success() {
            anyhow::bail!(
                "`aida mcp-serve` exited with {}\n{}",
                out.status,
                String::from_utf8_lossy(&out.stderr).trim()
            );
        }
        std::fs::write(workspace.join(PROBE), &out.stdout)
            .with_context(|| format!("writing {PROBE}"))?;
        Ok(())
    }
}
