# Add UBX-MON-SPAN message support

Closes #199

## Summary
Implements support for parsing UBX-MON-SPAN (0x0A 0x31) messages.

## Changes
- Add `MonSpan` packet definition with variable-length spectrum blocks
- Add `MonSpanBlock` struct and iterator for per-block access
- Register packet in protocol versions 27 and 31
- Add proptest-based fuzz tests

## Message Format
Variable length: 4 + N×272 bytes (header + spectrum blocks)

### Header
| Field        | Type | Description           |
|--------------|------|-----------------------|
| version      | u8   | Message version       |
| numRfBlocks  | u8   | Number of RF blocks   |

### Per Block (272 bytes)
| Field    | Type      | Description              |
|----------|-----------|--------------------------|
| spectrum | [u8; 256] | Spectrum bins            |
| span     | u32       | Frequency span (Hz)      |
| res      | u32       | Resolution (Hz)          |
| center   | u32       | Center frequency (Hz)    |
| pga      | u8        | PGA gain                 |
