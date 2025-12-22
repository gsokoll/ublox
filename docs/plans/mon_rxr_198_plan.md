# MON-RXR Implementation Plan (Issue #198)

## Message Specification
- **Class**: 0x0A (MON)
- **ID**: 0x21
- **Payload Length**: 1 byte (fixed)
- **Description**: Receiver Status Information - sent when receiver changes from or to backup mode
- **Supported**:
  - u-blox 6: yes
  - u-blox 7: yes
  - u-blox 8/M8: protocol versions 15-23.01
  - u-blox F9: protocol version 27.x
  - u-blox M10: protocol version 34.x (SPG 5.x firmware)

### Fields
| Offset | Name   | Type | Description                                      |
|--------|--------|------|--------------------------------------------------|
| 0      | flags  | X1   | Receiver status flags                            |

### flags Bitfield
| Bit | Name  | Description                |
|-----|-------|----------------------------|
| 0   | awake | not in backup mode         |

## Implementation Steps

### 1. Create Message Definition
**File**: `ublox/src/ubx_packets/packets/mon_rxr.rs`

```rust
#[ubx_packet_recv]
#[ubx(class = 0x0a, id = 0x21, fixed_payload_len = 1)]
struct MonRxr {
    /// Receiver status flags
    /// Bit 0: awake - receiver is awake and not in backup mode
    #[ubx(map_type = RxrFlags)]
    flags: u8,
}
```

Define `RxrFlags` enum/bitflags for the flags field with helper methods.

### 2. Register in Protocol Versions
Add to all protocol versions:
- `packetref_proto14.rs` (M8)
- `packetref_proto23.rs` (M8)
- `packetref_proto27.rs` (F9)
- `packetref_proto31.rs` (F9/M10)

### 3. Create Fuzz Test
**File**: `ublox/tests/fuzz_mon_rxr.rs`

Following the NAV-PVT pattern:
- Define `MonRxr` payload struct
- Implement `to_bytes()` serialization
- Create `mon_rxr_payload_strategy()` for proptest
- Create `ubx_mon_rxr_frame_strategy()` to generate valid UBX frames
- Add proptest that parses generated frames and verifies field values

### 4. Unit Tests
- Test parsing of a known valid MON-RXR message
- Test flags accessor methods

## Files to Create/Modify
1. **Create**: `ublox/src/ubx_packets/packets/mon_rxr.rs`
2. **Modify**: `ublox/src/ubx_packets/packets.rs` - add module declaration
3. **Modify**: `packetref_proto27.rs`, `packetref_proto31.rs` - register packet
4. **Create**: `ublox/tests/fuzz_mon_rxr.rs`

## Testing
```bash
cargo test --features "ubx_proto27" mon_rxr
cargo test --features "ubx_proto27" fuzz_mon_rxr
```
