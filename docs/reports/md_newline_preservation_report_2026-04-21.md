# Markdown Newline Preservation Report (2026-04-21)

## Summary

- Changed `devkit md` editing paths to preserve the existing file newline style instead of always inserting LF.
- Kept `devkit encoding normalize --newline lf|crlf` as the explicit whole-file repair and normalization command.
- Added regression tests covering CRLF Markdown edits.

## Changed Files

- `rust/crates/devkit-md/src/lib.rs`
- `README.md`
- `docs/reports/md_newline_preservation_report_2026-04-21.md`

## What Changed

- Added newline-style detection based on the existing file content.
- Normalized inserted section content, generated headings, and appended bullets to the detected newline style.
- Left unchanged lines untouched so local Markdown edits no longer create mixed newlines just by using `devkit md ...`.

## Verification

- Ran `cargo test -p devkit-md`.
- Ran `cargo run -p devkit-cli -- md append-section ...` style verification indirectly through the crate behavior covered by the new tests.
- Ran `cargo run -p devkit-cli -- encoding check README.md docs/reports/md_newline_preservation_report_2026-04-21.md --brief`.

## Remaining Risk

- Mixed-newline source files still need `devkit encoding normalize` when the goal is to repair the entire file to a single style.
- The detection rule currently prefers CRLF if the file contains any CRLF; that is appropriate for stable files, but a deliberately mixed file still needs explicit normalization.
