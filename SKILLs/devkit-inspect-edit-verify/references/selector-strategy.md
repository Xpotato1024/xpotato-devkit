# Selector Strategy

Use this reference when choosing how to narrow the edit surface.

## Selector order

1. Heading or markdown marker
   - Best for docs and mixed text files
2. Symbol or function
   - Best for code with stable structure
3. Outline then context
   - Use when the exact symbol or section name is not yet certain
4. Line range
   - Use only when higher-level selectors are unavailable

## Typical loop

1. `devkit diff summarize`
2. `devkit block outline <file>`
3. `devkit block context <file> <function>`
4. `devkit patch diagnose <patch-file>`
5. `devkit patch apply <patch-file>`
6. `devkit diff summarize`
