# Add UBX-MON-RXBUF message support

Closes #197

## Summary
Implements support for parsing UBX-MON-RXBUF (0x0A 0x07) messages.

## Changes
- Add `MonRxbuf` packet definition with buffer statistics arrays
- Register packet in protocol versions 27 and 31
- Add proptest-based fuzz tests

## Message Format
Fixed 24-byte payload with per-target statistics for 6 I/O targets.

| Field     | Type     | Description                    |
|-----------|----------|--------------------------------|
| pending   | [u16; 6] | Bytes pending per target       |
| usage     | [u8; 6]  | Buffer usage % per target      |
| peakUsage | [u8; 6]  | Peak usage % per target        |
