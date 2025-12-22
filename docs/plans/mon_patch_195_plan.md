# MON-PATCH Implementation Plan (Issue #195)

## Message Specification
- **Class**: 0x0A (MON)
- **ID**: 0x27
- **Payload Length**: Variable (4 + nEntries*16 bytes)
- **Description**: Installed Patches Information
- **Supported**: 
  - u-blox 8/M8: protocol versions 15-23.01
  - u-blox F9 (ZED-F9P): protocol version 27.x (HPG 1.x firmware)
  - u-blox M10: protocol version 34.x (SPG 5.x firmware)

### Header Fields
| Offset | Name     | Type | Description                           |
|--------|----------|------|---------------------------------------|
| 0      | version  | U2   | Message version (0x0001 for this ver) |
| 2      | nEntries | U2   | Total number of reported patches      |

### Per-Entry Fields (16 bytes each, repeated nEntries times)
| Offset   | Name             | Type | Description                            |
|----------|------------------|------|----------------------------------------|
| 4+n*16   | patchInfo        | X4   | Status info about the reported patch   |
| 8+n*16   | comparatorNumber | U4   | The number of the comparator           |
| 12+n*16  | patchAddress     | U4   | Address targeted by the patch          |
| 16+n*16  | patchData        | U4   | Data inserted at the patchAddress      |

### patchInfo Bitfield
| Bits  | Name      | Description                                        |
|-------|-----------|---------------------------------------------------|
| 0     | activated | 1: patch is active, 0: otherwise                  |
| 2..1  | location  | Where patch is stored: 0=OTP, 1=ROM, 2=BBR, 3=FS  |

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
Add to all protocol versions:
- `packetref_proto14.rs` (M8)
- `packetref_proto23.rs` (M8)
- `packetref_proto27.rs` (F9)
- `packetref_proto31.rs` (F9/M10)

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
