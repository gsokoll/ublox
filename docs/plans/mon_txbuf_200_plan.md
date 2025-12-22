# MON-TXBUF Implementation Plan (Issue #200)

## Message Specification
- **Class**: 0x0A (MON)
- **ID**: 0x08
- **Payload Length**: 28 bytes (fixed)
- **Description**: Transmitter Buffer Status
- **Supported**:
  - u-blox 8/M8: protocol versions 15-23.01
  - u-blox F9: protocol version 27.x (deprecated, use MON-COMMS instead)
  - u-blox M10: protocol version 34.x (SPG 5.x firmware)
  - u-blox F10: protocol version 40.x

### Fields
| Offset | Name      | Type     | Description                              |
|--------|-----------|----------|------------------------------------------|
| 0      | pending   | U2[6]    | Number of bytes pending in transmitter buffer for each target |
| 12     | usage     | U1[6]    | Maximum usage transmitter buffer during last sysmon period (%) |
| 18     | peakUsage | U1[6]    | Maximum usage transmitter buffer for each target (%) |
| 24     | tUsage    | U1       | Maximum usage of transmitter buffer for all targets (%) |
| 25     | tPeakUsage| U1       | Maximum usage of transmitter buffer for all targets (%) |
| 26     | errors    | X1       | Error bitmask |
| 27     | reserved0 | U1       | Reserved |

### errors Bitfield
| Bits | Name  | Description |
|------|-------|-------------|
| 5..0 | limit | Buffer limit of corresponding target reached |
| 6    | mem   | Memory Allocation error |
| 7    | alloc | Allocation error (TX buffer full) |

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
Add to all protocol versions:
- `packetref_proto14.rs` (M8)
- `packetref_proto23.rs` (M8)
- `packetref_proto27.rs` (F9)
- `packetref_proto31.rs` (F9/M10)

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
