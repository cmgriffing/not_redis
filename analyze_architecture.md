# Architecture Analysis: not_redis

**Date:** 2026-04-05
**Scope:** Full structural analysis of the `not_redis` crate — a Redis-compatible in-memory data structure library for Rust.

---

## Table of Contents

1. [Project Overview](#1-project-overview)
2. [File Structure & Module Hierarchy](#2-file-structure--module-hierarchy)
3. [Dependency Analysis](#3-dependency-analysis)
4. [Core Architecture Patterns](#4-core-architecture-patterns)
5. [Type System Design](#5-type-system-design)
6. [Concurrency Model](#6-concurrency-model)
7. [Structural Issues](#7-structural-issues)
8. [Dead Code & Unused Modules](#8-dead-code--unused-modules)
9. [Inconsistent Patterns](#9-inconsistent-patterns)
10. [API Surface Analysis](#10-api-surface-analysis)
11. [Test Organization](#11-test-organization)
12. [CI/CD & Build Configuration](#12-cicd--build-configuration)
13. [Recommendations Summary](#13-recommendations-summary)

---

## 1. Project Overview

| Property | Value |
|----------|-------|
| Crate name | `not_redis` |
| Version | 0.5.0 |
| Edition | Rust 2021 |
| Total source lines | ~4,850 (excluding tests/benches) |
| Compilation status | **Compiles successfully** (`cargo check` passes) |
| Test status | **56 tests pass**, 2 doc-tests pass |

The project provides a Redis-compatible in-memory store that runs inside the application process, eliminating network overhead. It supports strings, hashes, lists, sets, sorted sets, and streams.

---

## 2. File Structure & Module Hierarchy

### File Layout

```
src/
├── lib.rs                     (1,941 lines) — Core: types, StorageEngine, Client, traits
├── main.rs                    (37 lines)    — Demo binary
├── client.rs                  (1,698 lines) — Commands trait + alternate Client (NOT compiled)
├── error.rs                   (28 lines)    — RedisError enum (NOT compiled)
├── bin/
│   └── hgetall_bench.rs       (92 lines)    — Benchmark binary
├── storage/
│   ├── mod.rs                 (12 lines)    — Module re-exports
│   ├── engine.rs              (336 lines)   — StorageEngine with MemoryTracker
│   ├── types.rs               (63 lines)    — RedisData, StoredValue
│   ├── config.rs              (90 lines)    — MaxMemoryPolicy, StorageConfig
│   ├── expire.rs              (87 lines)    — ExpirationManager
│   └── memory.rs              (252 lines)   — MemoryTracker
├── types/
│   ├── mod.rs                 (9 lines)     — Module re-exports
│   ├── value.rs               (65 lines)    — Value enum
│   ├── from_redis_value.rs    (126 lines)   — FromRedisValue trait
│   └── to_redis_args.rs       (114 lines)   — ToRedisArgs trait
tests/
└── integration_tests.rs       (579 lines)   — Integration tests
benches/
└── (13 benchmark files)
docs/
├── tutorial.md, how-to.md, reference.md, explanation.md
```

### Module Hierarchy (as compiled)

The crate has **no `mod` declarations** in `lib.rs` (other than `#[cfg(test)] mod tests`). This means:

- `src/storage/` — **Not compiled** as part of the library crate
- `src/types/` — **Not compiled** as part of the library crate
- `src/error.rs` — **Not compiled** as part of the library crate
- `src/client.rs` — **Not compiled** as part of the library crate

**Everything is defined inline in `lib.rs`** (1,941 lines). The `storage/`, `types/`, and `error.rs` modules exist on disk but are never referenced via `mod` declarations. They appear to be an earlier or alternative design that was superseded by the monolithic `lib.rs`.

---

## 3. Dependency Analysis

### Production Dependencies

| Crate | Version | Purpose | Notes |
|-------|---------|---------|-------|
| `tokio` | 1.0 | Async runtime | `features = ["full"]` — heavy; only needs `rt`, `time`, `sync` |
| `dashmap` | 6.0 | Concurrent hash map | Core data structure |
| `arc-swap` | 1.7 | Atomic Arc swapping | **Unused** — not referenced in compiled code |
| `thiserror` | 2.0 | Error derive macro | Used for `RedisError` |
| `rustc-hash` | 2.0 | Fast hash (FxHash) | Used for `FxHashMap`/`FxHashSet` |
| `rand` | 0.8 | Random number generation | **Unused** in compiled code (referenced in `storage/memory.rs` which isn't compiled) |
| `smallvec` | 1.11 | Stack-allocated small vectors | Used in `ToRedisArgs` return type in `lib.rs` |

### Dev Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `tokio` | 1.0 | `features = ["test-util"]` |
| `criterion` | 0.5 | Benchmarking (`features = ["async"]`) |
| `redis` | 0.27 | Baseline comparison in benchmarks |

### Dependency Issues

| Issue | Severity | Details |
|-------|----------|---------|
| **`arc-swap` unused** | Low | Listed in `[dependencies]` but never imported in compiled code |
| **`rand` unused** | Low | Only referenced in `storage/memory.rs` which isn't compiled into the crate |
| **`tokio` features = ["full"]** | Low | Over-broad feature set; only `rt`, `time`, `sync` are needed |

---

## 4. Core Architecture Patterns

### Storage Layer (`lib.rs` lines 202-597)

The `StorageEngine` is defined **directly in `lib.rs`**, not in `src/storage/engine.rs`:

```
StorageEngine {
    data: Arc<DashMap<String, StoredValue>>,    // Concurrent hash map
    expiration: ExpirationManager,              // TTL tracking
    high_water_mark: Arc<AtomicUsize>,          // Memory compaction trigger
}
```

Key design decisions:
- **DashMap** for concurrent read/write without global lock
- **Arc<RedisData>** inside `StoredValue` for copy-on-write via `Arc::make_mut()`
- **BTreeMap<Instant, FxHashSet<String>>** for time-ordered expiration sweeping
- **High-water mark** tracking for auto-compaction (triggers when len < 25% of peak)

### Client Layer (`lib.rs` lines 778-1402)

The `Client` wraps `StorageEngine` and provides async Redis-compatible methods:

```
Client {
    storage: StorageEngine,
}
```

All client methods are `pub async fn` even though the underlying storage operations are synchronous. This is intentional for API compatibility with `redis-rs`.

### Expiration Model

- **Passive expiration**: Checked on read (each `get`, `exists`, etc. checks `is_expired()`)
- **Active expiration**: Background Tokio task sweeps every 100ms
- **BTreeMap ordering**: Enables efficient range queries on expiration times

### Memory Compaction

- Tracks `high_water_mark` of maximum entries stored
- On `remove()`, if `current_len * 4 < high_water_mark`, triggers `shrink_to_fit()`
- Prevents unbounded memory from DashMap's internal allocation strategy

---

## 5. Type System Design

### Dual Type Layers

The crate has two distinct type systems:

1. **RESP types** (`Value` enum, `lib.rs:89-134`): Protocol-level representation
   - `Null`, `Int(i64)`, `String(Vec<u8>)`, `Array(Vec<Value>)`, `Map`, `Set`, `Bool`, `Okay`

2. **Storage types** (`RedisData` enum, `lib.rs:145-152`): Internal data representation
   - `String(Vec<u8>)`, `List(VecDeque)`, `Set(FxHashSet)`, `Hash(FxHashMap)`, `ZSet(BTreeMap)`, `Stream(Vec<StreamEntry>)`

### Conversion Traits

| Trait | Direction | Return Type (`lib.rs`) | Return Type (`types/` module) |
|-------|-----------|------------------------|-------------------------------|
| `ToRedisArgs` | Rust → Value | `SmallVec<[Value; 1]>` | `Vec<Value>` |
| `FromRedisValue` | Value → Rust | `RedisResult<T>` | `RedisResult<T>` |

**Inconsistency**: The compiled `ToRedisArgs` uses `SmallVec<[Value; 1]>` while the uncompiled version in `types/to_redis_args.rs` uses `Vec<Value>`.

---

## 6. Concurrency Model

| Component | Sync Primitive | Justification |
|-----------|---------------|---------------|
| Data store | `DashMap` (sharded RwLock) | High-throughput concurrent access |
| Expirations | `Arc<Mutex<BTreeMap>>` | Low contention (only touched on set/expire/sweep) |
| High-water mark | `AtomicUsize` | Single counter, lock-free |
| StoredValue.data | `Arc<RedisData>` | Copy-on-write for mutations |

The `Client` requires `&mut self` for all operations despite using thread-safe primitives underneath. This forces callers to wrap `Client` in `Arc<Mutex<Client>>` for shared access, which is a deliberate API choice matching redis-rs ergonomics (but somewhat undermines the thread-safety of the underlying `StorageEngine`).

---

## 7. Structural Issues

### Issue 1: Monolithic `lib.rs` (1,941 lines)

**Severity: Medium**

`lib.rs` contains everything: error types, RESP value types, conversion traits, `ExpirationManager`, `StorageEngine`, `Client`, and all command implementations. The module files exist on disk but are never compiled.

**Impact:**
- Harder to navigate and maintain
- No separation of concerns at the module level
- All types are in a single namespace

### Issue 2: Orphaned Module Files

**Severity: High**

The following files are **never compiled** because no `mod` declarations exist in `lib.rs`:

| File | Lines | Status |
|------|-------|--------|
| `src/error.rs` | 28 | Orphaned — `RedisError` redefined in `lib.rs` |
| `src/client.rs` | 1,698 | Orphaned — depends on non-existent `crate::commands` module |
| `src/storage/mod.rs` | 12 | Orphaned |
| `src/storage/engine.rs` | 336 | Orphaned — different `StorageEngine` impl |
| `src/storage/types.rs` | 63 | Orphaned — `RedisData` missing `Stream` variant, no `Arc` wrapper |
| `src/storage/expire.rs` | 87 | Orphaned — different `ExpirationManager` API |
| `src/storage/memory.rs` | 252 | Orphaned — `MemoryTracker` (not used in `lib.rs` version) |
| `src/storage/config.rs` | 90 | Orphaned — `MaxMemoryPolicy`, `StorageConfig` |
| `src/types/mod.rs` | 9 | Orphaned |
| `src/types/value.rs` | 65 | Orphaned — has `From<u64>` impl not in `lib.rs` |
| `src/types/from_redis_value.rs` | 126 | Orphaned — more complete (has `u64`, `isize`, `usize`, `()` impls) |
| `src/types/to_redis_args.rs` | 114 | Orphaned — uses `Vec<Value>` instead of `SmallVec`, has tuple macros |

This amounts to **~2,936 lines of dead code** sitting in the repository.

### Issue 3: `client.rs` References Non-Existent Module

**Severity: High (but not blocking — file isn't compiled)**

`src/client.rs:6` imports from a module that doesn't exist:

```rust
use crate::commands::{Cmd, SetOptions, IntegerReplyOrNoOp, CopyOptions, execute_command};
```

No `commands.rs` or `commands/` directory exists. This file would fail to compile if included.

### Issue 4: Duplicate `StorageEngine::new()` in `storage/engine.rs`

**Severity: Medium (file isn't compiled, but confusing)**

`src/storage/engine.rs` defines `StorageEngine::new()` twice (lines 25-27 and 38-40), which would cause a compilation error if the module were included.

### Issue 5: Type Divergence Between `lib.rs` and Module Files

**Severity: High**

The "same" types differ significantly between compiled and orphaned code:

| Type | `lib.rs` (compiled) | Module file (orphaned) |
|------|---------------------|------------------------|
| `RedisData` | Has `Stream` variant | Missing `Stream` variant |
| `StoredValue.data` | `Arc<RedisData>` | Plain `RedisData` (no Arc) |
| `StorageEngine` | Has `high_water_mark`, inline expiration | Has `MemoryTracker`, delegated expiration |
| `RedisError` | 5 variants | 8 variants (adds `CommandNotFound`, `InvalidArgument`, `IoError`) |
| `ToRedisArgs` return | `SmallVec<[Value; 1]>` | `Vec<Value>` |
| `FromRedisValue` for `String` | Handles `Null` → empty string | Handles `Okay` → "OK" |

---

## 8. Dead Code & Unused Modules

### Unused Dependencies

| Dependency | Evidence |
|------------|----------|
| `arc-swap` | `grep` for `arc_swap` or `ArcSwap` finds zero matches in `lib.rs` |
| `rand` | Only used in orphaned `storage/memory.rs` |

### Unreachable Code Paths

- `src/storage/` entire directory (840 lines) — never compiled
- `src/types/` entire directory (314 lines) — never compiled  
- `src/error.rs` (28 lines) — never compiled
- `src/client.rs` (1,698 lines) — never compiled

**Total orphaned code: ~2,880 lines (37% of all Rust source in the repo)**

---

## 9. Inconsistent Patterns

### Pattern 1: Expiration Checking

The `Client` methods manually check `is_expired()` and call `remove()` on every read operation. This is done inconsistently:

| Method | Checks expiration? | Location |
|--------|--------------------|----------|
| `get` | Yes | `lib.rs:816` |
| `exists` | Yes | `lib.rs:864-867` |
| `hset` | Yes | `lib.rs:931-934` |
| `hget` | Yes | `lib.rs:965-968` |
| `hgetall` | Yes | `lib.rs:994-996` |
| `hdel` | Yes | `lib.rs:1025-1029` |
| `lpush` | Yes | `lib.rs:1054-1057` |
| `rpush` | Yes | `lib.rs:1087-1090` |
| `llen` | Yes | `lib.rs:1119-1121` |
| `sadd` | Yes | `lib.rs:1142-1145` |
| `smembers` | Yes | `lib.rs:1172-1174` |
| `xadd` | **No** | — |
| `xlen` | **No** | — |
| `xrange` | **No** | — |
| `xrevrange` | **No** | — |

Stream operations (`xadd`, `xlen`, `xrange`, `xrevrange`) delegate to `StorageEngine` which does **not** check expiration. A stream key that has expired will still return data until the background sweeper runs.

### Pattern 2: Key Parameter Types

| Method | Key type | Pattern |
|--------|----------|---------|
| `get` | `K: Into<String>` | Generic Into |
| `set` | `K: Into<String>` | Generic Into |
| `del` | `K: ToRedisArgs` | Trait-based |
| `exists` | `K: ToRedisArgs` | Trait-based |
| `expire` | `K: ToRedisArgs` | Trait-based |
| `hset` | `K: Into<String>` | Generic Into |
| `hget` | `K: Into<String>` | Generic Into |
| `hgetall` | `K: ToRedisArgs` | Trait-based |
| `hdel` | `K: ToRedisArgs` | Trait-based |
| `lpush` | `K: Into<String>` | Generic Into |
| `rpush` | `K: Into<String>` | Generic Into |
| `llen` | `K: ToRedisArgs` | Trait-based |
| `persist` | `&str` | Direct reference |
| `xadd` | `K: ToRedisArgs` | Trait-based |

Three different patterns for accepting keys (`Into<String>`, `ToRedisArgs`, `&str`) — should be unified.

### Pattern 3: Direct Field Access vs. Method Calls

In several `Client` methods (e.g., `hset`, `hget`, `hgetall`, `hdel`, `smembers`), the code accesses `self.storage.data` directly:

```rust
if let Some(mut stored) = self.storage.data.get_mut(&key_str) { ... }
```

This bypasses the `StorageEngine` API and reaches into its private `data` field. Since `StorageEngine.data` is `pub(crate)` (or pub in the same crate), this compiles, but it violates encapsulation — the `StorageEngine` cannot track reads/writes that happen through direct field access.

### Pattern 4: Error Type Inconsistency

The compiled `RedisError` (in `lib.rs`) has 5 variants:
- `ParseError`, `NoSuchKey(String)`, `WrongType`, `NotSupported`, `Unknown(String)`

The orphaned `error.rs` has 8 variants (adds `CommandNotFound`, `InvalidArgument`, `IoError`).

Neither implements `From<std::io::Error>` or other standard conversions.

---

## 10. API Surface Analysis

### Public Exports

The crate exports the following at the top level (all from `lib.rs`):

| Export | Type | Should be public? |
|--------|------|-------------------|
| `RedisError` | enum | Yes |
| `RedisResult<T>` | type alias | Yes |
| `Value` | enum | Yes |
| `RedisData` | enum | Questionable — internal storage type |
| `StreamEntry` | type alias | Questionable — tied to internals |
| `StoredValue` | struct | Questionable — includes `Arc<RedisData>` |
| `StorageEngine` | struct | Yes — allows shared storage |
| `ToRedisArgs` | trait | Yes |
| `FromRedisValue` | trait | Yes |
| `Client` | struct | Yes |

`RedisData`, `StreamEntry`, and `StoredValue` expose internal implementation details. Users generally interact through `Client` and shouldn't need to construct `RedisData` directly.

### Missing Functionality

Commands present in `client.rs` (orphaned) but absent from the compiled `Client`:

- **Strings**: `APPEND`, `GETRANGE`, `SETRANGE`, `STRLEN`, `INCR`, `DECR`, `MGET`, `MSET`
- **Hashes**: `HMGET`, `HKEYS`, `HVALS`, `HLEN`, `HINCRBY`, `HEXISTS`
- **Lists**: `LPOP`, `RPOP`, `LRANGE`, `LINDEX`
- **Sets**: `SREM`, `SCARD`, `SISMEMBER`, `SINTER`, `SUNION`, `SDIFF`
- **Sorted Sets**: `ZADD`, `ZREM`, `ZCARD`, `ZSCORE`, `ZRANGE`
- **Bit operations**: `BITCOUNT`
- **Key management**: `KEYS`, `TYPE`, `RENAME`, `RENAMENX`, `UNLINK`, `COPY`
- **Server**: `FLUSHALL`, `LASTSAVE`, `TIME`

The compiled `Client` supports 22 commands vs. the orphaned `Commands` trait which declares ~80+.

---

## 11. Test Organization

### Test Distribution

| Location | Test count | Type |
|----------|-----------|------|
| `lib.rs` (unit tests) | 9 | Sync unit tests for `StorageEngine` |
| `tests/integration_tests.rs` | 47 | Async integration tests via `Client` |
| Doc-tests | 2 | Compile-only examples |
| **Total** | **58** | |

### Test Coverage Gaps

| Component | Has tests? | Notes |
|-----------|-----------|-------|
| `StorageEngine` basic ops | Partial | Only compaction and stream tests |
| `StorageEngine` expiration | No | Background sweeper not tested |
| `Client` string ops | Yes | Via integration tests |
| `Client` hash ops | Yes | Via integration tests |
| `Client` list ops | Yes | Via integration tests |
| `Client` set ops | Yes | Via integration tests |
| `Client` stream ops | No | No integration tests for xadd/xlen/etc. |
| `ToRedisArgs` conversions | No | No unit tests |
| `FromRedisValue` conversions | No | No unit tests |
| Expiration passive check | Partial | Tested for strings, not for all types |
| Type safety (WrongType errors) | No | No tests that incorrect type access returns WrongType |

### Test Patterns

- All integration tests use `#[tokio::test]`
- Setup helper creates client and calls `start()`
- Tests clean up via `flushdb()`
- Good use of typed return values: `let name: String = client.get("key").await?`

---

## 12. CI/CD & Build Configuration

### GitHub Actions Workflows

| Workflow | Triggers | Jobs |
|----------|----------|------|
| `ci.yml` | Push/PR to main | lint, test (multi-platform), coverage, deny |
| `security.yml` | Dependency security scanning | cargo-audit |
| `benchmark.yml` | PR only | Performance regression check |
| `enforce-pr.yml` | Push to main | Ensures changes go through PRs |

### Platform Matrix

| OS | Target |
|----|--------|
| ubuntu-latest | x86_64-unknown-linux-gnu |
| macos-latest | x86_64-apple-darwin |
| macos-latest | aarch64-apple-darwin |
| windows-latest | x86_64-pc-windows-msvc |

### Benchmark Configuration

- 13 benchmark files in `benches/`
- All use Criterion framework
- `redis_baseline.rs` compares against real Redis instance
- 5% regression threshold (configurable via CONTRIBUTING.md)

---

## 13. Recommendations Summary

### Critical (should address)

| # | Issue | Impact |
|---|-------|--------|
| 1 | **~2,880 lines of orphaned code** across `storage/`, `types/`, `error.rs`, `client.rs` | Confusion, maintenance burden, misleading structure |
| 2 | **Monolithic `lib.rs`** at 1,941 lines | Hard to navigate and maintain |
| 3 | **Unused dependencies** (`arc-swap`, `rand`) | Increases compile time and binary size |

### Medium (should plan to address)

| # | Issue | Impact |
|---|-------|--------|
| 4 | **Inconsistent key parameter types** (`Into<String>` vs `ToRedisArgs` vs `&str`) | Confusing API surface |
| 5 | **Direct `storage.data` access** from `Client` methods | Breaks encapsulation, prevents future optimization |
| 6 | **Stream commands skip expiration checks** | Expired stream keys return stale data |
| 7 | **Internal types (`RedisData`, `StoredValue`) publicly exported** | Leaks implementation details |

### Low (nice to have)

| # | Issue | Impact |
|---|-------|--------|
| 8 | **No tests for type conversion traits** | Risk of regression in `ToRedisArgs`/`FromRedisValue` |
| 9 | **No tests for stream operations** via `Client` | Stream commands untested at API level |
| 10 | **`tokio` features = ["full"]** | Pulls in more than needed |
| 11 | **Missing memory management in compiled `StorageEngine`** | The orphaned `MemoryTracker` has eviction policies; the compiled version has none |

### Architectural Decision: Monolith vs. Modules

The project is at a crossroads. Two parallel implementations exist:

**Option A — Keep monolithic `lib.rs`:** Delete the orphaned module files, keep the single-file approach. Appropriate for a simple library where the entire implementation is < 2,000 lines.

**Option B — Refactor into modules:** Extract the inline code from `lib.rs` into proper modules, integrate the additional functionality from the orphaned files (eviction policies, more commands, richer error types). This would be needed to grow the project sustainably.

Either way, the current state of having both is the worst of both worlds — it doubles the code surface area without any benefit.
