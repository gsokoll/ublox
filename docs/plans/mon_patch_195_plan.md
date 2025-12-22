# MON-PATCH Implementation Plan (Issue #195)

## Message Specification
- **Class**: 0x0A (MON)
- **ID**: 0x27
- **Payload Length**: Variable (4 + N*16 bytes)
- **Description**: Installed Patches Information

### Header Fields
| Offset | Name       | Type   | Description                    |
|--------|------------|--------|--------------------------------|
| 0      | version    | u16    | Message version                |
| 2      | nEntries   | u16    | Number of patch entries        |

### Per-Entry Fields (16 bytes each)
| Offset | Name       | Type | Description                      |
|--------|------------|------|----------------------------------|
| 0      | patchInfo  | u32  | Patch info flags                 |
| 4      | comparatorNumber | u32 | Comparator ID               |
| 8      | patchAddress | u32 | Address of patch location       |
| 12     | patchData  | u32  | Patch data (first word)          |

## Implementation Steps

### 1. Create Message Definition
**File**: `ublox/src/ubx_packets/packets/mon_patch.rs`

```rust
#[ubx_packet_recv]
#[ubx(class = 0x0a, id = 0x27, max_payload_len = 260)]
struct MonPatch {
    version: u16,
    n_entries: u16,
    #[ubx(map_type = MonPatchEntryIter,
          from = MonPatchEntryIter::new,
          size_fn = mon_patch_data_size)]
    data: [u8; 0],
}
```

Define `MonPatchEntry` struct and `PatchInfo` bitflags for patch info field.

### 2. Register in Protocol Versions
Add to `packetref_proto27.rs` and `packetref_proto31.rs`.

### 3. Create Fuzz Test
**File**: `ublox/tests/fuzz_mon_patch.rs`

- Generate variable number of entries (0-16)
- Create payload strategy for 16-byte entry blocks
- Verify parser handles variable-length correctly

### 4. Unit Tests
- Test parsing with 0, 1, and multiple entries
- Test patch info flag accessors

## Files to Create/Modify
1. **Create**: `ublox/src/ubx_packets/packets/mon_patch.rs`
2. **Modify**: `ublox/src/ubx_packets/packets.rs`
3. **Modify**: Protocol packetref files
4. **Create**: `ublox/tests/fuzz_mon_patch.rs`

## Testing
```bash
cargo test --features "ubx_proto27" mon_patch
cargo test --features "ubx_proto27" fuzz_mon_patch
```
