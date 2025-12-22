# MON-MSGPP Implementation Plan (Issue #194)

## Message Specification
- **Class**: 0x0A (MON)
- **ID**: 0x06
- **Payload Length**: 120 bytes (fixed)
- **Description**: Message Parse and Process Status

### Fields
| Offset | Name    | Type      | Description                                    |
|--------|---------|-----------|------------------------------------------------|
| 0      | msg1    | [u16; 8]  | Message counts for port 1 (8 protocol types)   |
| 16     | msg2    | [u16; 8]  | Message counts for port 2                      |
| 32     | msg3    | [u16; 8]  | Message counts for port 3                      |
| 48     | msg4    | [u16; 8]  | Message counts for port 4                      |
| 64     | msg5    | [u16; 8]  | Message counts for port 5                      |
| 80     | msg6    | [u16; 8]  | Message counts for port 6                      |
| 96     | skipped | [u32; 6]  | Skipped bytes per port                         |

Protocol indices: 0=UBX, 1=NMEA, 2=RTCM2, 3=RTCM3, 4-7=reserved

## Implementation Steps

### 1. Create Message Definition
**File**: `ublox/src/ubx_packets/packets/mon_msgpp.rs`

```rust
#[ubx_packet_recv]
#[ubx(class = 0x0a, id = 0x06, fixed_payload_len = 120)]
struct MonMsgpp {
    /// Message counts for port 1 (per protocol)
    msg1: [u16; 8],
    msg2: [u16; 8],
    msg3: [u16; 8],
    msg4: [u16; 8],
    msg5: [u16; 8],
    msg6: [u16; 8],
    /// Skipped bytes per port
    skipped: [u32; 6],
}
```

Add helper methods to access counts by port/protocol.

### 2. Register in Protocol Versions
Add to `packetref_proto27.rs` and `packetref_proto31.rs`.

### 3. Create Fuzz Test
**File**: `ublox/tests/fuzz_mon_msgpp.rs`

- Generate strategies for all arrays
- Verify round-trip parsing of 120-byte payload

### 4. Unit Tests
- Test parsing and accessor methods

## Files to Create/Modify
1. **Create**: `ublox/src/ubx_packets/packets/mon_msgpp.rs`
2. **Modify**: `ublox/src/ubx_packets/packets.rs`
3. **Modify**: Protocol packetref files
4. **Create**: `ublox/tests/fuzz_mon_msgpp.rs`

## Testing
```bash
cargo test --features "ubx_proto27" mon_msgpp
cargo test --features "ubx_proto27" fuzz_mon_msgpp
```
