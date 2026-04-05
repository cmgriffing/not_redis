# Dead Code Analysis Report

**Project:** not_redis v0.5.0  
**Date:** 2026-04-05  
**Scope:** Unused functions, unreachable branches, unused imports, stale modules

---

## 1. Stale Modules (Never Compiled) -- CRITICAL

`src/lib.rs` is a **monolithic file** that contains all types, traits, and implementations inline. It declares **no `mod` statements** (other than `#[cfg(test)] mod tests`). As a result, the following source files exist on disk but are **never included in compilation**:

### 1.1 `src/client.rs` (1,698 lines)

- Contains a separate `Commands` trait, `Client` struct, `ClientBuilder`, and `impl Commands for Client`
- Imports from a non-existent module `crate::commands` (line 6: `use crate::commands::{Cmd, SetOptions, IntegerReplyOrNoOp, CopyOptions, execute_command}`)
- **Would fail to compile** even if included, because `crate::commands` does not exist
- Contains `impl_command!` macro (lines 844-870) that is defined but never invoked
- The `Commands` trait has **duplicate method declarations** -- `set`, `set_options`, `mset`, `mget`, `del`, `exists`, `append`, `getrange`, `setrange`, `strlen`, `incr`, `decr`, `hget`, `hmget`, `hset`, `hdel`, `hgetall`, `hkeys`, `hvals`, `hlen`, `hincr`, `hexists` each appear twice (lines 21-211 with docs, lines 213-342 without docs)
- Has `Client::with_storage()` while lib.rs has `Client::from_storage()` -- naming inconsistency between the two implementations

### 1.2 `src/error.rs` (28 lines)

- Duplicate `RedisError` enum already defined in `lib.rs` (lines 67-78)
- Contains **extra variants** not present in lib.rs:
  - `CommandNotFound(String)` (line 17)
  - `InvalidArgument(String)` (line 19)
  - `IoError(String)` (line 21)
- These extra variants are never used anywhere

### 1.3 `src/storage/` directory (6 files, ~838 lines total)

| File | Lines | Description |
|------|-------|-------------|
| `mod.rs` | 12 | Module declarations and re-exports |
| `engine.rs` | 335 | Alternative `StorageEngine` with memory tracking, eviction, `MemoryTracker` integration |
| `types.rs` | 63 | Separate `RedisData` and `StoredValue` (without `Stream` variant, without `Arc` wrapper) |
| `config.rs` | 90 | `StorageConfig` struct, `MaxMemoryPolicy` enum (8 eviction policies) |
| `expire.rs` | 86 | Separate `ExpirationManager` with `start_sweep_task`, `cancel_expiration`, `schedule_expiration` |
| `memory.rs` | 252 | `MemoryTracker` with LRU/LFU/random/volatile-TTL eviction strategies |

Notable issues within these stale files:
- `memory.rs` imports `BinaryHeap` (line 3) and `AtomicU64` (line 4) -- neither is used even within the file
- `engine.rs` defines `StorageEngine::new()` **twice** -- a no-arg version (line 25) and one taking `sweep_interval_ms: u64` (line 38), both with the same function name
- `storage/types.rs` defines `RedisData` without `Stream` variant (diverged from lib.rs)
- `storage/types.rs` defines `StoredValue` with `data: RedisData` (not `Arc<RedisData>` as in lib.rs)

### 1.4 `src/types/` directory (4 files, ~314 lines total)

| File | Lines | Description |
|------|-------|-------------|
| `mod.rs` | 9 | Module declarations and re-exports |
| `value.rs` | 65 | Duplicate `Value` enum (identical to lib.rs definition, plus extra `From<u64>`) |
| `from_redis_value.rs` | 126 | Duplicate `FromRedisValue` trait with different implementations (handles `Value::Okay`, adds `u64`/`isize`/`usize`/`()` impls) |
| `to_redis_args.rs` | 114 | Duplicate `ToRedisArgs` trait returning `Vec<Value>` instead of `SmallVec<[Value; 1]>`, adds `&[u8]`, `Vec<T>`, and tuple impls |

**Total stale code: ~2,878 lines across 12 files**

---

## 2. Unused Enum Variants

### 2.1 `Value` enum (lib.rs, lines 89-98)

| Variant | Line | Status | Notes |
|---------|------|--------|-------|
| `Value::Map(Vec<(Value, Value)>)` | 94 | **UNUSED** | Never constructed, never matched against anywhere |
| `Value::Set(Vec<Value>)` | 95 | **UNUSED** | Never constructed, never matched against anywhere |
| `Value::Okay` | 97 | **UNUSED** | Never constructed in lib.rs; only handled in stale `types/from_redis_value.rs` |

### 2.2 `RedisData` enum (lib.rs, lines 145-152)

| Variant | Line | Status | Notes |
|---------|------|--------|-------|
| `RedisData::ZSet(BTreeMap<Vec<u8>, f64>)` | 150 | **UNUSED** | Never constructed. No sorted set commands are implemented in `Client` (lib.rs). The `BTreeMap` import at line 55 exists only for this variant. |

### 2.3 `RedisError` enum (lib.rs, lines 67-78)

| Variant | Line | Status | Notes |
|---------|------|--------|-------|
| `RedisError::NotSupported` | 74 | **UNUSED** | Only used in stale `client.rs` (for `bit_and`, `bit_or`, `bit_xor`, `bit_not` stubs) |
| `RedisError::Unknown(String)` | 76 | **UNUSED** | Never constructed anywhere |
| `RedisError::NoSuchKey(String)` | 70 | **UNUSED** | Never constructed anywhere |

---

## 3. Unused/Unreferenced Public Functions and Methods

### 3.1 `StorageEngine` (lib.rs)

| Method | Line | Status | Notes |
|--------|------|--------|-------|
| `is_empty()` | 355-357 | **Unused externally** | Never called from `Client` or any external consumer. Defined for completeness but not needed by any command implementation. |
| `compact()` | 324-328 | **Public but internal-only** | Only called from private `maybe_compact()` (line 337) and tests. Could be `pub(crate)` or private. |

### 3.2 `StreamEntry` type alias (lib.rs, line 137)

```rust
pub type StreamEntry = (Vec<u8>, Vec<(Vec<u8>, Vec<u8>)>);
```
- Defined as a public type alias but only used internally within lib.rs (in `xrange`, `xrevrange`, `stream_entries_to_value`). Not part of any public Client API signature.

---

## 4. Unreachable Branches

### 4.1 `Client::value_to_vec()` (lib.rs, lines 1384-1395)

```rust
fn value_to_vec<V: ToRedisArgs>(v: &V) -> Vec<u8> {
    let args = v.to_redis_args();
    for arg in args {
        match arg {
            Value::String(s) => return s,
            Value::Int(n) => return n.to_string().into_bytes(),
            Value::Bool(b) => return (if b { "1" } else { "0" }).to_string().into_bytes(),
            _ => {}  // <-- handles Null, Array, Map, Set, Okay
        }
    }
    Vec::new()
}
```

- The `_ => {}` arm handles `Value::Null`, `Value::Array`, `Value::Map`, `Value::Set`, and `Value::Okay`
- `Value::Map` and `Value::Set` are never constructed (see Section 2.1), making those sub-cases permanently unreachable
- All current `ToRedisArgs` implementations in lib.rs only produce `String`, `Int`, `Bool`, or `Null` -- `Array` is never produced by `to_redis_args()`, making that sub-case also unreachable

### 4.2 `FromRedisValue for bool` (lib.rs, lines 726-739)

```rust
Value::String(s) => {
    let s_str = String::from_utf8(s).map_err(|_| RedisError::ParseError)?;
    Ok(s_str == "1" || s_str.eq_ignore_ascii_case("true"))
}
```

- No Client method in lib.rs ever returns `Value::String` where a `bool` return type is expected. The bool conversion from string is defined but the code path is currently unreachable through normal Client API usage.

---

## 5. Unused Dependencies

### 5.1 `arc-swap = "1.7"` (Cargo.toml, line 16)

- **Not imported or used anywhere** in the compiled library code (`lib.rs`)
- Was likely used by the stale `storage/` module (though it's not imported there either)
- Pure dead dependency

### 5.2 `rand = "0.8"` (Cargo.toml, line 19)

- **Not used by the library crate** (`lib.rs`)
- Only used in `src/bin/hgetall_bench.rs` (line 80: `rand::random::<u32>()`)
- Could be moved to `[dev-dependencies]` or scoped to the binary

---

## 6. Unused Imports Within Stale Files

These are in files that are themselves dead code, but noted for completeness:

| File | Import | Status |
|------|--------|--------|
| `src/storage/memory.rs:3` | `std::collections::BinaryHeap` | Imported but never used |
| `src/storage/memory.rs:4` | `std::sync::atomic::AtomicU64` | Imported but never used |
| `src/storage/memory.rs:1` | `crate::storage::config::StorageConfig` | Used in field type but whole file is dead |

---

## 7. Divergence Between Compiled and Stale Implementations

The stale modules represent a **more advanced architecture** that was never wired into `lib.rs`:

| Feature | `lib.rs` (compiled) | Stale modules (not compiled) |
|---------|---------------------|------------------------------|
| Memory eviction | None | LRU, LFU, Random, Volatile-TTL, Volatile-LRU, Volatile-LFU, Volatile-Random |
| Configuration | Hardcoded | `StorageConfig` with builder pattern |
| Client builder | None | `ClientBuilder` with `maxmemory()`, `maxmemory_policy()` |
| `RedisData` variants | 6 (String, List, Set, Hash, ZSet, Stream) | 5 (no Stream) |
| `StoredValue.data` type | `Arc<RedisData>` | `RedisData` (no Arc) |
| `ToRedisArgs` return type | `SmallVec<[Value; 1]>` | `Vec<Value>` |
| `FromRedisValue` impls | String, Vec<u8>, i64, bool, Value, Vec<T>, Option<T> (missing) | Adds u64, isize, usize, () |
| Command dispatch | Direct method calls | `Cmd` + `execute_command()` pattern |
| Error variants | 5 | 7 (adds CommandNotFound, InvalidArgument, IoError) |

---

## 8. Summary

| Category | Count | Lines |
|----------|-------|-------|
| Stale modules (never compiled) | 12 files | ~2,878 |
| Unused enum variants | 6 variants | -- |
| Unused public methods | 2 methods | -- |
| Unreachable branches | 2 locations | -- |
| Unused dependencies | 2 crates | -- |
| Unused imports (in stale files) | 2 imports | -- |

### Recommendations (not applied)

1. **Decide on architecture**: Either integrate the modular `src/storage/`, `src/types/`, `src/error.rs`, `src/client.rs` files by adding `mod` declarations to lib.rs, or remove them entirely. Currently they are dead weight.
2. **Remove unused `Value` variants**: `Map`, `Set`, `Okay` if not planned for future use.
3. **Remove unused `RedisData::ZSet`** or implement sorted set commands in Client.
4. **Remove unused `RedisError` variants**: `NotSupported`, `Unknown`, `NoSuchKey`.
5. **Remove `arc-swap` dependency** from Cargo.toml.
6. **Move `rand` to `[dev-dependencies]`** or scope it to the bench binary.
