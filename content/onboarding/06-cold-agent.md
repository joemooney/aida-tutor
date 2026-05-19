## Step 6 — the cold start

This is the part a skeptic can't wave away. Everything so far, they'd say,
is `git log --grep` with extra steps. Watch what that misses.

In Claude Code, start a completely fresh session:

```
/clear
```

That agent now has **zero memory** of everything you just did — no
conversation, no context, a blank slate. Now ask it:

    What does FR-1 cover in this project, and is it finished?

It answers correctly: the feature, the flag, the implementing commit, the
status. Not because you told it — you didn't, you cleared the chat. It
answers because `aida init` wired up `.mcp.json`, and the fresh agent read
the project's AIDA store directly.

That is the whole idea. Your context never lived in the conversation,
where it dies when the session closes. It lived in the project, where the
next agent — and the one after that — picks it up cold.

You closed the session. The next agent picked up exactly where you left
off, because the project remembered, not the chat.
