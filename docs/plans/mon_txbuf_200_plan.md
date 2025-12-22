# MON-TXBUF Implementation Plan (Issue #200)

## Message Specification
- **Class**: 0x0A (MON)
- **ID**: 0x08
- **Payload Length**: 28 bytes (fixed)
- **Description**: Transmitter Buffer Status

### Fields
| Offset | Name      | Type     | Description                              |
|--------|-----------|----------|------------------------------------------|
| 0      | pending   | [u16; 6] | Bytes pending per target (6 targets)     |
| 12     | tUsage    | [u8; 6]  | TX buffer usage % per target             |
| 18     | tPeakUsage| [u8; 6]  | Peak TX buffer usage % per target        |
| 24     | errors    | u8       | Error flags                              |
| 25     | reserved1 | u8       | Reserved                                 |
| 26     | limit     | u8       | Max usage configuration                  |
| 27     | reserved2 | u8       | Reserved                                 |

## Implementation Steps

### 1. Create Message Definition
**File**: `ublox/src/ubx_packets/packets/mon_txbuf.rs`

```rust
#[ubx_packet_recv]
#[ubx(class = 0x0a, id = 0x08, fixed_payload_len = 28)]
struct MonTxbuf {
    /// Bytes pending per target
    pending: [u16; 6],
    /// TX buffer usage percentage per target
    t_usage: [u8; 6],
    /// Peak TX buffer usage percentage per target
    t_peak_usage: [u8; 6],
    /// Error flags
    #[ubx(map_type = TxbufErrors)]
    errors: u8,
    reserved1: u8,
    /// Maximum usage configuration
    limit: u8,
    reserved2: u8,
}
```

Define `TxbufErrors` bitflags for error field.

### 2. Register in Protocol Versions
Add to `packetref_proto27.rs` and `packetref_proto31.rs`.

### 3. Create Fuzz Test
**File**: `ublox/tests/fuzz_mon_txbuf.rs`

- Generate strategies for the 6-element arrays
- Test error flags parsing
- Verify round-trip parsing

### 4. Unit Tests
- Test parsing of known valid MON-TXBUF message
- Test error flag accessors

## Files to Create/Modify
1. **Create**: `ublox/src/ubx_packets/packets/mon_txbuf.rs`
2. **Modify**: `ublox/src/ubx_packets/packets.rs`
3. **Modify**: Protocol packetref files
4. **Create**: `ublox/tests/fuzz_mon_txbuf.rs`

## Testing
```bash
cargo test --features "ubx_proto27" mon_txbuf
cargo test --features "ubx_proto27" fuzz_mon_txbuf
```
