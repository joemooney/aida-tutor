## Goal

Enter a *role* — a persistent named hat that scopes your work — and see
how it attaches to your shell session.

## Why

Real projects aren't worked from one undifferentiated seat. You wear
different hats: planning and capturing work (the `dialog` hat),
heads-down coding (`implementer`), reviewing a diff (`reviewer`).

AIDA makes those hats first-class. A **role** is a persistent named
context. It does two things:

1. **Scopes your queue.** The work queue (exercises 22-24) is filtered
   by your active role — `aida queue next` shows you what's routed to
   the hat you're wearing, not everyone's pile.
2. **Marks your shell.** The active role prefixes your prompt and
   exports `AIDA_SESSION_ROLE`, so you (and `aida`) always know which
   hat is on.

That second point is the catch: **a role lives in your shell session,
not on disk.** That's why `aida role enter` doesn't just *do* something
— it prints shell code you have to `eval`. The `eval` is what lets it
reach into your shell to set the variable and the prompt.

## What to do

From `workspace/`, first see what roles exist:

```
aida role list
```

If the list is empty, install the starter set:

```
aida role scaffold
```

Now enter the `implementer` role — note the `eval`:

```
eval "$(aida role enter implementer)"
```

Your prompt gains a `(role:implementer)` prefix. Run `aida statusline`
and you'll see the role there too.

## Verify

`aida-tutor verify` — passes once a role is active in your shell.

Because a role is shell-scoped, **run `verify` in the same terminal**
where you ran the `eval`. The tutor reads the role from the shell it
was launched in — if you entered the role in one terminal and run
`verify` in another, the second terminal never saw the `eval`. That
isn't a bug in the tutor; it's the whole lesson of this exercise.

## Tip

`eval "$(aida role end)"` takes the hat off again. The state is
preserved — `aida role enter` later resumes right where you left off,
activity log and all.

## What's next

Exercise 22 — wear the producer hat and route work into a role's queue.
