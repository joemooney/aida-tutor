## You use a coding agent. Give it a memory.

Your coding agent is brilliant and completely forgetful. Every session
starts cold: it has never seen this project, doesn't know why a function
exists, can't tell you which decisions are already settled. So you
re-explain the same context, every time.

AIDA fixes that by giving the *project itself* a memory — a small,
queryable record of intent, decisions, and the links between specs and the
code that implements them. It lives in the repo. Any agent, in any
session, can read it.

In the next 15 minutes you'll feel the difference firsthand. You'll take a
real feature request, capture it, have your agent build it, commit — and
then watch a brand-new agent session, with zero memory of your
conversation, answer a question about that feature correctly. Not because
you told it. Because the project remembered.

## The scratch project

`workspace/` has been seeded with a tiny Python CLI called `greet`. It says
hello; that is all it does. Take a look:

```
cd workspace
cat greet.py
python3 greet.py World
```

It is deliberately small — this tour is about AIDA, not about the code.
You will add exactly one feature to it.
