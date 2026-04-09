# Metrics Interpretation Guide

Use this reference when reporting `devkit metrics show`.

## What to highlight

- Highest-frequency commands
- Slowest average commands
- Commands with lower success rate
- Whether brief mode dominates usage

## What not to overclaim

- Metrics show local usage, not user intent.
- High average time does not by itself prove a regression.
- Missing or empty metrics files should be reported as missing data, not as healthy zero activity.
