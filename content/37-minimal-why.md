<!-- trace:STORY-56 | ai:codex -->

# Markdown-first projects and `aida why`

Not every project is ready for a git-canonical store. AIDA's minimal first
run gives a folder of markdown specs plus a tiny traced example, so you can
experience the code-to-intent loop in about a minute.

From a temporary directory under `workspace/`, run:

```bash
aida init --minimal
aida why example.py:2
```

The `why` command follows the `# trace:EXAMPLE-1` comment into
`specs/EXAMPLE-1.md`. This path is intentionally lightweight. When the
project needs shared IDs, relationships, roles, or MCP, return to the project
root and run the full `aida init` path.
