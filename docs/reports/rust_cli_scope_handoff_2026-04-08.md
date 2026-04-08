# Rust CLI Scope Handoff

Date: 2026-04-08

## Summary

Documented the current Rust CLI command surface to remove PR blockers caused by public documentation overstating implemented functionality.

## Blockers Addressed In This Session

- Clarified the exact command groups currently supported by the Rust binary in `README.md`.
- Marked older broader capability descriptions as legacy / planned scope rather than current Rust parity.
- Kept Windows installer messaging explicit about being Windows-only.
- Kept `--time` and `--time-json` implemented and documented as currently supported global flags.

## Remaining Tasks For A Later Session

1. Rust parity for `encoding`
   - `devkit encoding check`
   - `devkit encoding normalize`
2. Rust parity for legacy support commands
   - `devkit bootstrap`
   - `devkit metrics`
3. Block command interface parity
   - `--symbol`
   - `--list-headings`
   - `--list-functions`
   - any other legacy selection helpers still referenced by docs
4. Patch command interface parity
   - align Rust CLI shape with documented `--patch-file` / `--dry-run` expectations
5. Public-doc cleanup
   - review README and design docs for any remaining references to unsupported Rust commands or flags

## Notes

- This handoff intentionally prioritizes removing misleading public claims over expanding implementation scope.
- The next session should decide whether to implement missing commands in Rust or further narrow public documentation.
