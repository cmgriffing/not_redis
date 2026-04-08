# Future Optimization Ideas

## SmallVec for Inline Storage (High Potential)
- **Concept**: Replace `Vec<u8>` with `SmallVec<[u8; 16]>` for stored byte buffers and hash/set collections. This avoids heap allocation for small strings (keys, values, fields) and improves cache locality.
- **Effort**: High — requires changing `RedisData` enum, all conversions, and many client methods.
- **Expected gain**: Potentially 10-30% more throughput depending on allocation patterns.
- **Status**: Not started; Idea discovered but not implemented during the current session.

## Other ideas
- (None yet)
