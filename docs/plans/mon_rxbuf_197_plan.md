# MON-RXBUF Implementation Plan (Issue #197)

## Message Specification
- **Class**: 0x0A (MON)
- **ID**: 0x07
- **Payload Length**: 24 bytes (fixed)
- **Description**: Receiver Buffer Status
- **Supported**:
  - u-blox 8/M8: protocol versions 15-23.01
  - u-blox F9: protocol version 27.x (deprecated, use MON-COMMS instead)
  - u-blox M10: protocol version 34.x (SPG 5.x firmware)

### Fields
| Offset | Name      | Type     | Description                              |
|--------|-----------|----------|------------------------------------------|
| 0      | pending   | U2[6]    | Number of bytes pending in receiver buffer for each target |
| 12     | usage     | U1[6]    | Maximum usage receiver buffer during last sysmon period (%) |
| 18     | peakUsage | U1[6]    | Maximum usage receiver buffer for each target (%) |

## Implementation Steps

### 1. Create Message Definition
**File**: `ublox/src/ubx_packets/packets/mon_rxbuf.rs`

```rust
#[ubx_packet_recv]
#[ubx(class = 0x0a, id = 0x07, fixed_payload_len = 24)]
struct MonRxbuf {
    /// Number of bytes pending in receiver buffer for each target (6 x U2 = 12 bytes)
    pending: [u8; 12],
    /// Maximum usage receiver buffer during last sysmon period for each target (%)
    usage: [u8; 6],
    /// Maximum usage receiver buffer for each target (%)
    peak_usage: [u8; 6],
}
```

Note: `pending` uses `[u8; 12]` since the macro requires byte arrays. Accessor methods needed for u16 values.

### 2. Register in Protocol Versions
Add to all protocol versions:
- `packetref_proto14.rs` (M8)
- `packetref_proto23.rs` (M8)
- `packetref_proto27.rs` (F9)
- `packetref_proto31.rs` (F9/M10)

### 3. Create Fuzz Test
**File**: `ublox/tests/fuzz_mon_rxbuf.rs`

- Generate strategies for the 6-element arrays
- Verify round-trip parsing

### 4. Unit Tests
- Test parsing of known valid MON-RXBUF message

## Files to Create/Modify
1. **Create**: `ublox/src/ubx_packets/packets/mon_rxbuf.rs`
2. **Modify**: `ublox/src/ubx_packets/packets.rs`
3. **Modify**: Protocol packetref files
4. **Create**: `ublox/tests/fuzz_mon_rxbuf.rs`

## Testing
```bash
cargo test --features "ubx_proto27" mon_rxbuf
cargo test --features "ubx_proto27" fuzz_mon_rxbuf
```
