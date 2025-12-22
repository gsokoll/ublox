# Add UBX-MON-RXR message support

Closes #198

## Summary
Implements support for parsing UBX-MON-RXR (0x0A 0x21) messages.

## Changes
- Add `MonRxr` packet definition with flags field
- Add `RxrFlags` type with `awake()` accessor
- Register packet in protocol versions 27 and 31
- Add proptest-based fuzz tests following NAV-PVT pattern

## Message Format
| Field  | Type | Description                |
|--------|------|----------------------------|
| flags  | u8   | Bit 0: awake (not backup)  |
