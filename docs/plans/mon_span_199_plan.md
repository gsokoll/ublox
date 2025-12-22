# MON-SPAN Implementation Plan (Issue #199)

## Message Specification
- **Class**: 0x0A (MON)
- **ID**: 0x31
- **Payload Length**: Variable (4 + N*34 bytes)
- **Description**: Signal Characteristics - RF spectrum analyzer for each RF path
- **Supported**:
  - u-blox F9: protocol version 27.x (HPG 1.x, LAP 1.x firmware)
  - u-blox M10: protocol version 34.x (SPG 5.x firmware)
  - u-blox F10: protocol version 40.x (SPG 6.x firmware)
  - NOT supported on u-blox 8/M8

### Header Fields
| Offset | Name      | Type | Description                     |
|--------|-----------|------|---------------------------------|
| 0      | version   | u8   | Message version (0x00)          |
| 1      | numRfBlocks| u8  | Number of RF blocks             |
| 2      | reserved1 | [u8;2]| Reserved                       |

### Per-Block Fields (34 bytes each)
| Offset | Name      | Type     | Description                     |
|--------|-----------|----------|---------------------------------|
| 0      | spectrum  | [u8; 256]| Spectrum data (256 bins)        |
| 256    | span      | u32      | Frequency span in Hz            |
| 260    | res       | u32      | Frequency resolution in Hz      |
| 264    | center    | u32      | Center frequency in Hz          |
| 268    | pga       | u8       | PGA gain                        |
| 269    | reserved  | [u8; 3]  | Reserved                        |

**Note**: Per-block size is 272 bytes (256 + 16), not 34. Total = 4 + N*272.

## Implementation Steps

### 1. Create Message Definition
**File**: `ublox/src/ubx_packets/packets/mon_span.rs`

```rust
#[ubx_packet_recv]
#[ubx(class = 0x0a, id = 0x31, max_payload_len = 1092)]
struct MonSpan {
    version: u8,
    num_rf_blocks: u8,
    reserved1: [u8; 2],
    #[ubx(map_type = MonSpanBlockIter,
          from = MonSpanBlockIter::new,
          size_fn = mon_span_data_size)]
    data: [u8; 0],
}
```

Define `MonSpanBlock` struct and iterator for spectrum blocks.

### 2. Register in Protocol Versions
Add to F9/M10 protocol versions only (not M8):
- `packetref_proto27.rs` (F9)
- `packetref_proto31.rs` (F9/M10)

### 3. Create Fuzz Test
**File**: `ublox/tests/fuzz_mon_span.rs`

- Generate variable number of blocks (1-4)
- Create payload strategy for spectrum data arrays
- Verify parser handles variable-length correctly

### 4. Unit Tests
- Test parsing with 1 and 2 blocks
- Test spectrum data accessor

## Files to Create/Modify
1. **Create**: `ublox/src/ubx_packets/packets/mon_span.rs`
2. **Modify**: `ublox/src/ubx_packets/packets.rs`
3. **Modify**: Protocol packetref files
4. **Create**: `ublox/tests/fuzz_mon_span.rs`

## Testing
```bash
cargo test --features "ubx_proto27" mon_span
cargo test --features "ubx_proto27" fuzz_mon_span
```
