---
name: devkit-git-drafts
description: Use when an AI agent needs to draft commit messages, PR bodies, or perform a guarded push with devkit.
---

# devkit git drafts

Use `devkit` when the task is about Git-facing text or a safe push.

## When to use

- A commit message should be drafted from a scoped diff.
- A PR body should be generated from the current change set.
- A push should be gated by safety checks.

## Workflow

1. Choose the right diff scope.
   - Prefer `--staged` for mixed worktrees.
   - Use `--base/--head` or `--commits` when the scope must be explicit.
2. Draft the text.
   - `devkit git commit-message --staged`
   - `devkit git pr-body --base <ref> --head <ref>`
   - Pass `--lang ja` or `--lang en` when the target language matters.
3. Push only after confirming branch safety.
   - `devkit git safe-push --remote <name> --yes`
   - Keep the safety checks intact even when confirmation is disabled.

## Rules

- Do not draft from an empty diff.
- Do not infer repository-specific policy from the tool output.
- Keep the output as plain text or Markdown so it can be reused directly.
- Prefer file output or machine-oriented flags from upstream inspection commands when another tool will consume the result, because default terminal output may be colorized for humans.

## Reference

- See [AI agent workflow](../../docs/design/ai_agent_workflow.md) for the inspect/edit/verify loop that feeds these Git helpers.
