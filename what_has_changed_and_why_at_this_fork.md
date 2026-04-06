# What Has Changed and Why — At This Fork

Changes made on the **[choas/not_redis](https://github.com/choas/not_redis)** fork compared to the upstream **[cmgriffing/not_redis](https://github.com/cmgriffing/not_redis)** repository.

The upstream project built the core not_redis library (22 Redis-compatible commands, DashMap-based concurrency, Criterion benchmarks, CI/CD pipelines, and automated performance research). This fork contributed **13 commits** adding performance optimizations, comprehensive documentation and analysis, code quality improvements, and testing infrastructure.

---

## 1. Performance Optimizations (commit `493edea`)

A single commit delivering four algorithmic improvements to the core library:

### 1a. ExpirationManager Reverse Index

**What changed:** Added a `key_to_time: FxHashMap<String, Instant>` reverse index to ExpirationManager, alongside the existing `time_to_keys: BTreeMap<Instant, FxHashSet<String>>`.

**Why:** Cancelling a key's expiration (on overwrite or PERSIST) previously required scanning all entries in the BTreeMap — O(n) in the number of scheduled expirations. The reverse index makes this O(1). This matters when many keys have TTLs.

### 1b. Atomic Entry Counter

**What changed:** Replaced `DashMap::len()` with an `AtomicUsize` counter that is incremented on insert and decremented on delete.

**Why:** `DashMap::len()` iterates all shards and briefly locks each one — O(shards) with lock overhead. The atomic counter provides O(1) DBSIZE queries with zero locking. This is especially important under high concurrency.

### 1c. Stream Operation Optimizations

**What changed:**
- XRANGE/XREVRANGE use `.take(count)` to short-circuit iteration instead of collecting all entries and truncating
- XREVRANGE iterates in reverse directly instead of collecting forward and reversing
- XDEL uses `FxHashSet<&[u8]>` for O(1) ID lookups instead of `Vec::contains()` which is O(n)

**Why:** These are algorithmic improvements. The old XRANGE would scan the entire stream even when only 10 entries were requested. The old XDEL performed O(n*m) comparisons (n stream entries times m IDs to delete). These changes reduce both time and memory complexity.

### 1d. Single-Lookup Mutations

**What changed:** HSET, HDEL, LPUSH, RPUSH, SADD now use a single `get_mut()` call to both check for expiration and perform the mutation.

**Why:** Previously, these operations performed one lookup to check if the key existed/was expired, then a second lookup to mutate it. Combining them into one halves the shard lock acquisitions.

---

## 2. Documentation — Diataxis Framework (commit `1ee50f9`)

**What changed:** Created four documentation files following the Diataxis framework:

- **`docs/tutorial.md`** — Step-by-step guide for new users, covering client creation, all data types, error handling, and a complete session management example
- **`docs/how-to.md`** — Task-oriented recipes: caching, session stores, async task communication, streams as event logs, rate limiting, visitor tracking
- **`docs/reference.md`** — Complete API reference with tables for all commands, types, traits, and error variants
- **`docs/explanation.md`** — Architecture rationale: why not_redis exists, concurrency model, memory management, performance analysis, and comparison with similar crates

**Why:** The upstream project had no user-facing documentation. Good documentation serves four distinct needs: learning (tutorials), doing (how-to guides), looking up (reference), and understanding (explanation). The Diataxis framework addresses all four, making the library accessible to newcomers while providing depth for advanced users.

---

## 3. Code Quality Analysis

### 3a. Architecture Analysis (commit `ddaa000`)

**What changed:** Created `analyze_architecture.md` — a 499-line structural analysis of the entire crate, covering module organization, dependency usage, type system design, concurrency patterns, test coverage, and 7 identified structural issues.

**Why:** Understanding the architecture is a prerequisite for making informed improvements. The analysis revealed that the project has ~2,878 lines of orphaned code in `src/storage/`, `src/types/`, `src/client.rs`, and `src/error.rs` that are not compiled into the library (all logic lives in `src/lib.rs`). This informs future refactoring decisions.

### 3b. Security Audit (commit `e99bef4`)

**What changed:** Created `analyze_secrets.md` documenting security findings:
- **Critical:** `.beads/.beads-credential-key` committed to git history
- **High:** `Mutex::lock().unwrap()` can panic on poisoned mutexes
- **Medium:** Unchecked integer casts and potential allocation overflows
- **Low:** Benchmark code uses `.unwrap()` without context

**Why:** Security issues compound over time. Documenting them ensures they're tracked and addressed, even if not fixed immediately.

### 3c. Dead Code Analysis (commit `b5905a3`)

**What changed:** Created `analyze_dead_code.md` identifying unused code: 2,878 lines of stale modules, unused enum variants, unused public methods, unreachable branches, and unused dependencies (`arc-swap`, `rand` in library code).

**Why:** Dead code increases maintenance burden and confuses new contributors. Documenting it is the first step toward cleanup.

### 3d. Performance Analysis (commit `7e6e36e`)

**What changed:** Created `analyze_performance.md` documenting all 6 optimizations from commit `493edea` with before/after complexity analysis and regression risk assessment.

**Why:** Performance changes need documentation so future contributors understand why the code is structured the way it is and don't accidentally revert optimizations.

### 3e. Project Overview (commit `ecf97a8`)

**What changed:** Created a detailed project overview and current state assessment documenting the project's structure, capabilities, and areas for improvement.

**Why:** Provides a baseline understanding of the project for contributors and serves as a starting point for prioritizing work.

---

## 4. Documentation Updates

### 4a. Contributing Guide Update (commit `7041c69`)

**What changed:** Updated `CONTRIBUTING.md` with benchmark enforcement details:
- No benchmark may regress by more than 5%
- Automated benchmark comparison on every PR
- Admin override process for justified regressions

**Why:** Clear contribution guidelines prevent performance regressions from being merged and set expectations for contributors.

### 4b. Documentation Review Updates (commit `0daa52e`)

**What changed:** Updated the Diataxis documentation files based on code review — correcting inaccuracies, improving examples, and aligning documentation with the actual API behavior.

**Why:** Initial documentation was written from the API surface; reviewing against actual code revealed discrepancies that needed correction.

### 4c. Redis Server Startup Instructions (commit `e219e76`)

**What changed:** Added instructions for starting a Redis server, needed when running the baseline comparison benchmarks (`benches/redis_baseline.rs`).

**Why:** The Redis baseline benchmarks require a running Redis server, but the project had no instructions for setting one up. Contributors trying to run `cargo bench --bench redis_baseline` would get connection errors without this guidance.

---

## 5. Testing Infrastructure (commit `2ebefa2`)

**What changed:** Created `run_integration_tests.sh` — a convenience script for running the integration test suite.

**Why:** Provides a simple one-command way to run all integration tests, lowering the barrier for contributors to verify their changes.

---

## 6. Lint Fixes and Code Quality (commit `1ee50f9`)

**What changed:** Fixed compiler lint warnings across the codebase (part of the same commit that added Diataxis documentation).

**Why:** Clean compiler output makes it easier to spot real issues. Warnings that accumulate without being addressed train contributors to ignore compiler output.

---

## 7. Dependency and Build Updates (commit `9cf92e4`)

**What changed:** Updated `Cargo.lock` with the `itoa` dependency.

**Why:** Ensures reproducible builds with the correct dependency versions locked.

---

## Summary

| Category | Commits | Impact |
|----------|---------|--------|
| Performance optimization | `493edea` | 4 algorithmic improvements: O(1) expiration cancel, O(1) DBSIZE, stream short-circuiting, halved lock acquisitions |
| Documentation (Diataxis) | `1ee50f9`, `0daa52e`, `e219e76` | 4 user-facing docs (tutorial, how-to, reference, explanation) + Redis setup instructions |
| Code analysis | `ecf97a8`, `ddaa000`, `e99bef4`, `b5905a3`, `7e6e36e` | Architecture review, security audit, dead code inventory, performance analysis |
| Code quality | `1ee50f9`, `9cf92e4` | Lint fixes, dependency lock updates |
| Testing | `2ebefa2` | Integration test runner script |
| Contributing | `7041c69` | Benchmark enforcement guidelines |
| Meta | `2012f10` | Comprehensive changelog (`what_has_changed_and_why.md`) |

---

## Complete Commit List

All 13 commits on the fork (by Lars Gregori), in reverse chronological order:

| Commit | Description |
|--------|-------------|
| `2012f10` | Document: write comprehensive changelog |
| `0daa52e` | Documentation: update docs based on code review |
| `e219e76` | Documentation: add Redis server startup instructions |
| `9cf92e4` | chore: update Cargo.lock with itoa dependency |
| `7e6e36e` | Analysis: performance improvement opportunities |
| `493edea` | **perf: optimize expiration management, entry counting, and stream operations** |
| `7041c69` | Documentation: update CONTRIBUTING.md with benchmark enforcement |
| `b5905a3` | Analysis: dead code inventory |
| `e99bef4` | Analysis: security audit |
| `ddaa000` | Analysis: architecture review |
| `1ee50f9` | **feat: fix lint warnings, add Diataxis documentation** |
| `2ebefa2` | Testing: integration test runner script |
| `ecf97a8` | Analysis: project overview and state assessment |
