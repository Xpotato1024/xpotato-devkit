# Doc Edit Command Patterns

Use this reference when a documentation task needs a concrete command choice.

## Common mappings

- Add a missing heading or section:
  - `devkit md ensure-section <file> <heading>`
- Replace the body under an existing heading:
  - `devkit md replace-section <file> <heading>`
- Append a new block below an existing heading:
  - `devkit md append-section <file> <heading>`
- Add one bullet under an existing heading:
  - `devkit md append-bullet <file> <heading> <bullet>`

## Note templates

- Implementation-oriented release or feature notes:
  - `devkit doc impl-note`
- Benchmark or performance notes:
  - `devkit doc benchmark-note`

## Verification pattern

1. `devkit encoding check <files> --brief`
2. `devkit block extract <file> --heading <heading>`
3. Re-run the encoding check after the change
