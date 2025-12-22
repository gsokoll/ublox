# Add UBX-MON-PATCH message support

Closes #195

## Summary
Implements support for parsing UBX-MON-PATCH (0x0A 0x27) messages.

## Changes
- Add `MonPatch` packet definition with variable-length patch entries
- Add `MonPatchEntry` struct and iterator for per-entry access
- Add `PatchInfo` bitflags for patch info field
- Register packet in protocol versions 27 and 31
- Add proptest-based fuzz tests

## Message Format
Variable length: 4 + N×16 bytes (header + patch entries)

### Header
| Field     | Type | Description           |
|-----------|------|-----------------------|
| version   | u16  | Message version       |
| nEntries  | u16  | Number of patches     |

### Per Entry (16 bytes)
| Field            | Type | Description           |
|------------------|------|-----------------------|
| patchInfo        | u32  | Patch info flags      |
| comparatorNumber | u32  | Comparator ID         |
| patchAddress     | u32  | Patch location        |
| patchData        | u32  | Patch data            |
