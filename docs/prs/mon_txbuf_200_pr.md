# Add UBX-MON-TXBUF message support

Closes #200

## Summary
Implements support for parsing UBX-MON-TXBUF (0x0A 0x08) messages.

## Changes
- Add `MonTxbuf` packet definition with buffer statistics arrays
- Add `TxbufErrors` bitflags for error field
- Register packet in protocol versions 27 and 31
- Add proptest-based fuzz tests

## Message Format
Fixed 28-byte payload with per-target statistics for 6 I/O targets.

| Field       | Type     | Description                    |
|-------------|----------|--------------------------------|
| pending     | [u16; 6] | Bytes pending per target       |
| tUsage      | [u8; 6]  | Buffer usage % per target      |
| tPeakUsage  | [u8; 6]  | Peak usage % per target        |
| errors      | u8       | Error flags                    |
| limit       | u8       | Max usage config               |
