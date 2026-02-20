# Contributing to not_redis

Thank you for your interest in contributing to not_redis! This document outlines our development workflow and requirements.

## Development Workflow

### 1. Create a Feature Branch

```bash
# Ensure you're on main and it's up to date
git checkout main
git pull origin main

# Create your feature branch
git checkout -b feature/my-awesome-feature
```

### 2. Make Your Changes

- Write clean, well-tested code
- Follow existing code style and conventions
- Add tests for new functionality
- Update documentation as needed

### 3. Run Local Checks

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Run tests
cargo test --all-features

# Run benchmarks (optional but recommended)
cargo bench --bench benchmarks
```

### 4. Commit and Push

```bash
git add .
git commit -m "feat: add my awesome feature"
git push origin feature/my-awesome-feature
```

### 5. Create a Pull Request

1. Go to GitHub and create a new Pull Request
2. Fill out the PR template with description of changes
3. Ensure all CI checks pass
4. Request review from maintainers

### 6. Address Review Feedback

- Respond to reviewer comments
- Make requested changes
- Push updates to the same branch

### 7. Merge

Once approved and all checks pass, a maintainer will merge your PR.

## Performance Requirements

All PRs must pass performance benchmarks comparing the PR branch against `main`:

- **Benchmark Environment**: Both main and PR code run on the same GitHub Actions runner instance
- **Clean Builds**: `cargo clean` runs between main and PR builds to ensure fair comparison
- **Regression Threshold**: No benchmark may be >5% slower than main
- **Comparison Method**: Criterion benchmarks with JSON output parsing

Benchmark results are automatically posted as a PR comment showing:
- Individual benchmark times (main vs PR)
- Percentage change for each benchmark
- Summary of regressions, improvements, and neutral results

## Admin Override for Benchmark Failures

In rare cases where a performance regression is acceptable (e.g., fixing a critical bug that requires slower code), repository admins can bypass the benchmark check:

### How to Override

1. Go to the PR page on GitHub
2. Click **"Merge pull request"** (it will be red/disabled due to failing checks)
3. Click the dropdown arrow next to the merge button
4. Select **"Merge without waiting for requirements to be met (bypass branch protections)"**
5. Provide a justification in the commit message explaining why the regression is acceptable

### When to Override

Only override benchmark failures for:
- Critical bug fixes that cannot be optimized further
- Security fixes that require performance trade-offs
- Changes to benchmark infrastructure that cause expected variance
- Cases where the regression is due to fixing incorrect behavior

**Note**: Always document the reason in the commit message when bypassing.

## Required GitHub Settings (Repository Admins)

To fully enforce this workflow, repository administrators must configure branch protection in GitHub Settings:

1. Go to **Settings** → **Branches** → **Branch protection rules**
2. Add rule for `main` branch with:
   - **Require a pull request before merging**: ✓ Checked
   - **Require approvals**: At least 1
   - **Dismiss stale PR approvals when new commits are pushed**: ✓ Checked
   - **Require status checks to pass**: ✓ Checked
     - Select: `lint`, `test`, `coverage`, `deny`
   - **Require branches to be up to date before merging**: ✓ Checked
   - **Include administrators**: ✓ Checked (this enforces PRs even for admins)

## Questions?

If you have questions about the contribution process, please:
- Open an issue for discussion
- Ask in your PR comments
- Contact the maintainers

Thank you for contributing!
