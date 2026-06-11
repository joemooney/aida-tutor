## Goal

Talk to AIDA over the Model Context Protocol — pipe a JSON-RPC request
into `aida mcp-serve` and capture the response.

## Why

This whole tutorial drove AIDA from the command line. But AIDA was
built to be driven by an *AI agent* — and an agent doesn't shell out to
`aida` and scrape text. It speaks **MCP**, the Model Context Protocol:
a JSON-RPC channel for tools.

Look back at exercise 01. When `aida init` ran, it wrote a `.mcp.json`
file at the repo root:

```
{
  "mcpServers": {
    "aida": {
      "type": "stdio",
      "command": "aida",
      "args": ["mcp-serve"]
    }
  }
}
```

That file tells Claude Code: "to work with requirements, launch
`aida mcp-serve` and talk to it over stdio." From then on the agent
reads, searches, and updates the store through structured tool calls —
`list_requirements`, `show_requirement`, `add_requirement`, and so on —
not by parsing CLI output.

You never run `aida mcp-serve` by hand; Claude Code manages it. But
running one request by hand, once, demystifies the whole thing.

## What to do

`aida mcp-serve` reads JSON-RPC 2.0 requests from stdin and writes
responses to stdout. From `workspace/`, send it one `tools/call`
request — invoking the `list_requirements` tool — and redirect the
reply into a file:

```
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"list_requirements","arguments":{}}}' | aida mcp-serve > mcp-probe.json
```

The server starts, answers the one request, sees end-of-input, and
stops. Open the reply:

```
cat mcp-probe.json
```

It's a JSON-RPC result: a `content` array whose text is the same
requirement list `aida list` prints — only now it arrived the way an
agent receives it. That round trip is every `/aida-*` command you've
used in this tutorial, underneath.

## Verify

`aida-tutor verify` — passes once `mcp-probe.json` exists and holds a
JSON-RPC `tools/call` reply: a `result` carrying a `content` array.

## Tip

`tools/call` is one of several methods. `tools/list` (no `params`)
makes `aida mcp-serve` enumerate every tool it exposes and the
arguments each one takes — the same list Claude Code reads on startup
to learn what it can do with your store.

## What's next

That's the whole tour — all 35 exercises. You started with an empty git
repo and `aida init`; you've walked capture, trace comments, AIDA-format
commits and the docs build, search and status, the two-leg push, the
distributed orphan store and its rebuildable cache, roles and the
producer/consumer queue, the requirement graph, scoped sessions with
worktrees, the code-review and commit-pairing workflow, and finally
plan linting, store auditing, and the MCP bridge.

Run `aida-tutor progress` to see 35/35. Now go run `aida init` in a
project of your own and capture its first real requirement.
