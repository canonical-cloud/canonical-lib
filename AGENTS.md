# Agent instructions

## Shared-history safety

Preserve concurrent and reviewed work. Before synchronizing, inspect/fetch the
remote and integrate upstream changes with `git merge` or `git pull`; do not use
`git rebase` to rewrite shared history. Do not use `git stash` to hide work that
may belong to another session, destructive `git reset` or `git clean`, or a
force-push / `git push --force` to bypass review or protected-branch history.
Use explicit additive commits and resolve conflicts semantically from both sides.

## Repository-local Git worktrees

- Create or use a Git worktree only when the human operator explicitly authorizes it for the current task. Concurrency or a dirty checkout is not permission by itself.
- Put every authorized worktree at `<repository-root>/tmp/worktrees/<name>`; from the repository root, use `./tmp/worktrees/<name>`. Never place worktrees beside repositories or organization directories.
- Keep `tmp`, `temp`, `tmp/worktrees`, and `temp/worktrees` ignored in the repository-root `.gitignore`. Do not commit files from those directories.
- Relocate or remove a worktree only when the operator explicitly requests it. Before removal, preserve and publish intended changes, verify its commit is represented on the target branch, and confirm there are no tracked, untracked, ignored-sensitive, or in-use files that must survive. Remove it with `git worktree remove <path>` without `--force`; never delete a worktree directory with `rm`.
