# Diff Scope Guide

Use this reference when the right diff scope is unclear.

## Preferred scope order

1. `--staged`
   - Use when the worktree is mixed and only staged changes should drive the draft.
2. `--base` and `--head`
   - Use when the draft should reflect a branch or PR comparison.
3. `--commits`
   - Use when the change set is defined by explicit commit range.

## Typical commands

- Commit message from staged work:
  - `devkit git commit-message --staged`
- PR body from branch comparison:
  - `devkit git pr-body --base origin/main --head HEAD`
- Safe push after verification:
  - `devkit git safe-push --remote origin --yes`
