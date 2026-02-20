# Agent Instructions

This repo is an in-process in-memory implementation of redis that adheres to the redis-rs api as close as possible.

## Branching Strategy

Every new feature, fix, or performance enhancement—basically anything—must be done in a new branch and submitted as a pull request.

### PR Requirements
- All changes to `main` must go through a Pull Request (no direct pushes)
- Branch protection enforces PR-only workflow
- CI checks (lint, test, coverage, security) must pass
- Performance benchmarks must show no >5% regression
- Admin override available for benchmark failures (documented in CONTRIBUTING.md)

See CONTRIBUTING.md for detailed workflow instructions.
