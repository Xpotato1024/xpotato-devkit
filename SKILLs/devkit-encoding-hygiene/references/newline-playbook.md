# Newline Playbook

Use this reference when newline normalization is the main risk.

## Recommended flow

1. Detect:
   - `devkit encoding check <files> --brief`
2. Preview if needed:
   - `devkit encoding normalize <files> --newline lf --dry-run`
   - `devkit encoding normalize <files> --newline crlf --dry-run`
3. Normalize:
   - `devkit encoding normalize <files> --newline lf`
   - `devkit encoding normalize <files> --newline crlf`
4. Verify:
   - `devkit encoding check <files> --brief`

## Decision rule

- Use the repository's established newline style for touched files.
- For Windows-facing docs that already use CRLF, preserve CRLF.
- Do not expand the normalization scope beyond the files relevant to the task.
