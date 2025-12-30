# Schema-Driven Code Generation

This directory contains tools for generating ublox-rs packet definitions and fuzz tests from validated UBX message schemas.

## Overview

Instead of manually writing packet definitions for each UBX message type, this approach:

1. **Reads validated JSON schemas** from [ubx-protocol-schema](https://github.com/gsokoll/ubx-protocol-schema)
2. **Generates Rust packet structs** with `#[ubx_packet_recv]` macros
3. **Generates proptest fuzz tests** with round-trip assertions

## Benefits

- **Accuracy**: Schemas are extracted from official u-blox PDFs and validated across 25+ interface manuals
- **Consistency**: All generated code follows the same patterns
- **Coverage**: 209 validated message definitions available
- **Maintainability**: Update schema, regenerate code
- **Fuzz testing**: Automatic proptest strategies for every message type

## Usage

```bash
# List available messages
python codegen/generate_from_schema.py --schema-dir codegen/schema --list

# Generate a specific message
python codegen/generate_from_schema.py --schema-dir codegen/schema --message MON-RXBUF --output codegen/generated

# Generate multiple messages
for msg in MON-RXBUF MON-TXBUF MON-SPAN NAV-PL; do
  python codegen/generate_from_schema.py --schema-dir codegen/schema --message "$msg" --output codegen/generated
done
```

## Generated Output

For each message (e.g., `MON-RXBUF`), generates:

### Packet Definition (`packets/mon_rxbuf.rs`)
```rust
#[ubx_packet_recv]
#[ubx(class = 0x0a, id = 0x07, fixed_payload_len = 24)]
struct MonRxbuf {
    pending: [u16; 6],
    usage: [u8; 6],
    peak_usage: [u8; 6],
}
```

### Fuzz Test (`tests/fuzz_mon_rxbuf.rs`)
```rust
fn mon_rxbuf_strategy() -> impl Strategy<Value = ExpectedMonRxbuf> { ... }
fn build_mon_rxbuf_frame(expected: &ExpectedMonRxbuf) -> Vec<u8> { ... }

proptest! {
    #[test]
    fn test_mon_rxbuf_roundtrip((expected, frame) in mon_rxbuf_frame_strategy()) {
        // Parse and verify round-trip
    }
}
```

## Schema Source

The `schema/` directory contains validated message definitions from [ubx-protocol-schema](https://github.com/gsokoll/ubx-protocol-schema).

To update schemas:
```bash
git clone https://github.com/gsokoll/ubx-protocol-schema.git /tmp/schema
cp /tmp/schema/data/validated/messages/*.json codegen/schema/validated/messages/
```

## Integration with ublox-rs

Generated packet definitions can be integrated into `ublox/src/ubx_packets/packets/`.
Generated fuzz tests can be added to `ublox/tests/`.

See the `generated/` directory for examples.
