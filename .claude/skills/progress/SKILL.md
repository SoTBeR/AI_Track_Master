---
name: progress
description: 'Show milestone progress from docs/tasks/, git branches and log, and recommend the next step.'
disable-model-invocation: true
allowed-tools: Read Grep Glob Bash(git log *) Bash(git branch *) Bash(git status *)
---

# Progress

Current branch: !`git branch --show-current`

1. Read the frontmatter (`id`, `status`, `depends_on`, `layer`) of every `docs/tasks/*/*.md` (skip README.md).
2. Run `git branch --list 'task/*'`, `git log --oneline -15 main`, `git status --short`.
3. Report to the maintainer in Russian, compactly:
   - per milestone: done / in-review / in-progress / todo / blocked counts;
   - tasks in-review (branch awaiting merge), in-progress, blocked with reason;
   - milestones from `docs/PLAN.md` section 9 without task files;
   - next step: the first `todo` task whose dependencies are done, as a ready command `/task <path>`,
     or `/plan-milestone <M#>` if the current milestone has no tasks.
