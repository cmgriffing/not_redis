
// Hypothesis: Avoid unnecessary Runtime::block_on calls in hot paths (set, get, remove) by using synchronous paths where possible or reducing the need to check memory status every time.

// Implementation:
// 1. In `set`, if memory is not enabled, skip the `block_on` for `is_enabled`.
// 2. In `get`, same for `is_enabled`. 
// 3. In `remove`, same for `is_enabled`.
// Note: We need to check if we can check `is_enabled` synchronously.
// Looking at `MemoryTracker` implementation (will check after).

// Actually, even better:
// The `set` method performs heavy `block_on` even if memory is disabled.
// If we can assume memory is either globally enabled or disabled, we can avoid the check.
// But since it's dynamic, let's see if we can at least minimize the `block_on`.

// Another idea:
// In `set`, the `old_value` is obtained via `self.data.get(key).cloned()`.
// This clones the `StoredValue`, which contains an `Arc<RedisData>`. 
// While cloning the Arc is cheap, the `StoredValue` itself is cloned.
// If we can avoid cloning if we don't need it.
