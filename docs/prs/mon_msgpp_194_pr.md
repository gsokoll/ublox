# Add UBX-MON-MSGPP message support

Closes #194

## Summary
Implements support for parsing UBX-MON-MSGPP (0x0A 0x06) messages.

## Changes
- Add `MonMsgpp` packet definition with per-port message statistics
- Register packet in protocol versions 27 and 31
- Add proptest-based fuzz tests

## Message Format
Fixed 120-byte payload with message parse counts per port and protocol.

| Field   | Type      | Description                          |
|---------|-----------|--------------------------------------|
| msg1-6  | [u16; 8]  | Message counts per protocol per port |
| skipped | [u32; 6]  | Skipped bytes per port               |

Protocol indices: 0=UBX, 1=NMEA, 2=RTCM2, 3=RTCM3
