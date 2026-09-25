<!-- trace:STORY-57 | ai:codex -->

# Graph context and focus

An individual spec is useful; its neighborhood is where planning context
lives. Run `aida graph FR-1 --tree` to inspect the parent/child rollup and
related context. Then set a temporary namespace:

```bash
aida focus FR-1
aida list
aida focus
```

While focus is set, supported reads stay inside that subtree and print a loud
header so the narrowed context is not silent. Clear it when you need the whole
project again:

```bash
aida focus --clear
```
