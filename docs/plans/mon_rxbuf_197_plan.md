# MON-RXBUF Implementation Plan (Issue #197)

## Message Specification
- **Class**: 0x0A (MON)
- **ID**: 0x07
- **Payload Length**: 24 bytes (fixed)
- **Description**: Receiver Buffer Status

### Fields
| Offset | Name      | Type     | Description                              |
|--------|-----------|----------|------------------------------------------|
| 0      | pending   | [u16; 6] | Bytes pending per target (6 targets)     |
| 12     | usage     | [u8; 6]  | RX buffer usage % per target             |
| 18     | peakUsage | [u8; 6]  | Peak RX buffer usage % per target        |

## Implementation Steps

### 1. Create Message Definition
**File**: `ublox/src/ubx_packets/packets/mon_rxbuf.rs`

```rust
#[ubx_packet_recv]
#[ubx(class = 0x0a, id = 0x07, fixed_payload_len = 24)]
struct MonRxbuf {
    /// Bytes pending per target
    pending: [u16; 6],
    /// RX buffer usage percentage per target
    usage: [u8; 6],
    /// Peak RX buffer usage percentage per target
    peak_usage: [u8; 6],
}
```

### 2. Register in Protocol Versions
Add to `packetref_proto27.rs` and `packetref_proto31.rs`.

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
