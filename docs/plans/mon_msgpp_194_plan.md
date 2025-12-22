# MON-MSGPP Implementation Plan (Issue #194)

## Message Specification
- **Class**: 0x0A (MON)
- **ID**: 0x06
- **Payload Length**: 120 bytes (fixed)
- **Description**: Message Parse and Process Status
- **Supported**: All u-blox generations (8/M8, F9, M10) - deprecated but still available

### Fields
| Offset | Name    | Type      | Description                                      |
|--------|---------|-----------|--------------------------------------------------|
| 0      | msg1    | U2[8]     | Successfully parsed messages per protocol, port0 |
| 16     | msg2    | U2[8]     | Successfully parsed messages per protocol, port1 |
| 32     | msg3    | U2[8]     | Successfully parsed messages per protocol, port2 |
| 48     | msg4    | U2[8]     | Successfully parsed messages per protocol, port3 |
| 64     | msg5    | U2[8]     | Successfully parsed messages per protocol, port4 |
| 80     | msg6    | U2[8]     | Successfully parsed messages per protocol, port5 |
| 96     | skipped | U4[6]     | Number of skipped bytes for each port            |

Protocol indices: 0=UBX, 1=NMEA, 2=RTCM2, 3=RTCM3, 4-7=reserved

## Implementation Steps

### 1. Create Message Definition
**File**: `ublox/src/ubx_packets/packets/mon_msgpp.rs`

```rust
#[ubx_packet_recv]
#[ubx(class = 0x0a, id = 0x06, fixed_payload_len = 120)]
struct MonMsgpp {
    /// Successfully parsed messages per protocol, port0 (8 x U2 = 16 bytes)
    msg1: [u8; 16],
    /// Successfully parsed messages per protocol, port1
    msg2: [u8; 16],
    /// Successfully parsed messages per protocol, port2
    msg3: [u8; 16],
    /// Successfully parsed messages per protocol, port3
    msg4: [u8; 16],
    /// Successfully parsed messages per protocol, port4
    msg5: [u8; 16],
    /// Successfully parsed messages per protocol, port5
    msg6: [u8; 16],
    /// Number of skipped bytes for each port (6 x U4 = 24 bytes)
    skipped: [u8; 24],
}
```

Note: The macro only supports `[u8; N]` arrays, so accessor methods are needed
to read u16/u32 values from the raw bytes.

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
