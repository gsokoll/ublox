//! A proptest generator for U-Blox MON-SPAN messages.
//!
//! This module provides a `proptest` strategy to generate byte-level
//! UBX frames containing a MON-SPAN message with variable RF blocks.

use byteorder::{LittleEndian, WriteBytesExt};
use proptest::prelude::*;
use ublox::{ParserBuilder, UbxPacket};

/// Represents a single RF spectrum block (272 bytes).
#[derive(Debug, Clone)]
pub struct SpanBlock {
    pub spectrum: [u8; 256],
    pub span: u32,
    pub res: u32,
    pub center: u32,
    pub pga: u8,
}

impl SpanBlock {
    fn to_bytes(&self) -> Vec<u8> {
        let mut wtr = Vec::with_capacity(272);
        wtr.extend_from_slice(&self.spectrum);
        wtr.write_u32::<LittleEndian>(self.span).unwrap();
        wtr.write_u32::<LittleEndian>(self.res).unwrap();
        wtr.write_u32::<LittleEndian>(self.center).unwrap();
        wtr.push(self.pga);
        wtr.extend_from_slice(&[0u8; 3]); // reserved1
        wtr
    }
}

/// Represents the payload of a UBX-MON-SPAN message.
#[derive(Debug, Clone)]
pub struct MonSpan {
    pub version: u8,
    pub blocks: Vec<SpanBlock>,
}

impl MonSpan {
    fn to_bytes(&self) -> Vec<u8> {
        let mut wtr = Vec::with_capacity(4 + self.blocks.len() * 272);
        wtr.push(self.version);
        wtr.push(self.blocks.len() as u8);
        wtr.extend_from_slice(&[0u8; 2]); // reserved0
        for block in &self.blocks {
            wtr.extend(block.to_bytes());
        }
        wtr
    }
}

/// Strategy for generating a single spectrum block.
fn span_block_strategy() -> impl Strategy<Value = SpanBlock> {
    (
        prop::collection::vec(any::<u8>(), 256), // spectrum as vec, convert to array
        any::<u32>(),                             // span
        any::<u32>(),                             // res
        any::<u32>(),                             // center
        any::<u8>(),                              // pga
    )
        .prop_map(|(spectrum_vec, span, res, center, pga)| {
            let mut spectrum = [0u8; 256];
            spectrum.copy_from_slice(&spectrum_vec);
            SpanBlock {
                spectrum,
                span,
                res,
                center,
                pga,
            }
        })
}

/// Strategy for generating a MonSpan payload with 1-2 blocks.
fn mon_span_payload_strategy() -> impl Strategy<Value = MonSpan> {
    (
        Just(0x00u8), // version is always 0x00
        prop::collection::vec(span_block_strategy(), 1..=2),
    )
        .prop_map(|(version, blocks)| MonSpan { version, blocks })
}

/// Calculates the 8-bit Fletcher-16 checksum used by U-Blox.
fn calculate_checksum(data: &[u8]) -> (u8, u8) {
    let mut ck_a: u8 = 0;
    let mut ck_b: u8 = 0;
    for byte in data {
        ck_a = ck_a.wrapping_add(*byte);
        ck_b = ck_b.wrapping_add(ck_a);
    }
    (ck_a, ck_b)
}

/// Strategy that generates a complete, valid UBX frame containing a MON-SPAN message.
pub fn ubx_mon_span_frame_strategy() -> impl Strategy<Value = (MonSpan, Vec<u8>)> {
    mon_span_payload_strategy().prop_map(|span| {
        let payload = span.to_bytes();
        let class_id = 0x0a;
        let message_id = 0x31;
        let length = payload.len() as u16;

        let mut frame_core = Vec::with_capacity(4 + payload.len());
        frame_core.push(class_id);
        frame_core.push(message_id);
        frame_core.write_u16::<LittleEndian>(length).unwrap();
        frame_core.extend_from_slice(&payload);

        let (ck_a, ck_b) = calculate_checksum(&frame_core);

        let mut final_frame = Vec::with_capacity(8 + payload.len());
        final_frame.push(0xB5);
        final_frame.push(0x62);
        final_frame.extend_from_slice(&frame_core);
        final_frame.push(ck_a);
        final_frame.push(ck_b);

        (span, final_frame)
    })
}

#[cfg(feature = "ubx_proto27")]
proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))] // Reduced cases due to large payloads
    #[test]
    fn test_parser_proto27_with_generated_mon_span_frames(
        (expected, frame) in ubx_mon_span_frame_strategy()
    ) {
        use ublox::proto27::{Proto27, PacketRef};

        let mut parser = ParserBuilder::new().with_protocol::<Proto27>().with_fixed_buffer::<2048>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto27(PacketRef::MonSpan(p)))) = it.next() else {
            panic!("Parser failed to parse a MON-SPAN valid packet");
        };

        // Verify header fields
        prop_assert_eq!(p.version(), expected.version);
        prop_assert_eq!(p.num_rf_blocks(), expected.blocks.len() as u8);

        // Verify blocks via iterator
        let parsed_blocks: Vec<_> = p.blocks().collect();
        prop_assert_eq!(parsed_blocks.len(), expected.blocks.len());

        for (i, (parsed, exp)) in parsed_blocks.iter().zip(expected.blocks.iter()).enumerate() {
            prop_assert_eq!(parsed.spectrum, exp.spectrum, "Block {} spectrum mismatch", i);
            prop_assert_eq!(parsed.span, exp.span, "Block {} span mismatch", i);
            prop_assert_eq!(parsed.res, exp.res, "Block {} res mismatch", i);
            prop_assert_eq!(parsed.center, exp.center, "Block {} center mismatch", i);
            prop_assert_eq!(parsed.pga, exp.pga, "Block {} pga mismatch", i);
        }
    }
}
