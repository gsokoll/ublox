//! A proptest generator for U-Blox MON-RXBUF messages.
//!
//! This module provides a `proptest` strategy to generate byte-level
//! UBX frames containing a MON-RXBUF message.

use byteorder::{LittleEndian, WriteBytesExt};
use proptest::prelude::*;
use ublox::{ParserBuilder, UbxPacket};

/// UBX Sync Character 1 (0xB5 = 'µ')
const SYNC_CHAR_1: u8 = 0xB5;
/// UBX Sync Character 2 (0x62 = 'b')
const SYNC_CHAR_2: u8 = 0x62;

/// Represents the payload of a UBX-MON-RXBUF message.
#[derive(Debug, Clone)]
pub struct MonRxbuf {
    /// Number of bytes pending in receiver buffer for each target (6 targets)
    pub pending: [u16; 6],
    /// Maximum usage receiver buffer during last sysmon period for each target (%)
    pub usage: [u8; 6],
    /// Maximum usage receiver buffer for each target (%)
    pub peak_usage: [u8; 6],
}

impl MonRxbuf {
    fn to_bytes(&self) -> Vec<u8> {
        let mut wtr = Vec::with_capacity(24);
        for p in &self.pending {
            wtr.write_u16::<LittleEndian>(*p).unwrap();
        }
        wtr.extend_from_slice(&self.usage);
        wtr.extend_from_slice(&self.peak_usage);
        wtr
    }
}

/// Strategy for generating a MonRxbuf payload.
fn mon_rxbuf_payload_strategy() -> impl Strategy<Value = MonRxbuf> {
    (
        prop::array::uniform6(any::<u16>()),
        prop::array::uniform6(0u8..=100u8), // usage is percentage
        prop::array::uniform6(0u8..=100u8), // peak_usage is percentage
    )
        .prop_map(|(pending, usage, peak_usage)| MonRxbuf {
            pending,
            usage,
            peak_usage,
        })
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

/// Strategy that generates a complete, valid UBX frame containing a MON-RXBUF message.
pub fn ubx_mon_rxbuf_frame_strategy() -> impl Strategy<Value = (MonRxbuf, Vec<u8>)> {
    mon_rxbuf_payload_strategy().prop_map(|rxbuf| {
        let payload = rxbuf.to_bytes();
        let class_id = 0x0a;
        let message_id = 0x07;
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

        (rxbuf, final_frame)
    })
}

#[cfg(feature = "ubx_proto27")]
proptest! {
    #[test]
    fn test_parser_proto27_with_generated_mon_rxbuf_frames(
        (expected, frame) in ubx_mon_rxbuf_frame_strategy()
    ) {
        use ublox::proto27::{Proto27, PacketRef};

        let mut parser = ParserBuilder::new().with_protocol::<Proto27>().with_fixed_buffer::<1024>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto27(PacketRef::MonRxbuf(p)))) = it.next() else {
            panic!("Parser failed to parse a MON-RXBUF valid packet");
        };

        // Verify parsed values match expected using accessor methods
        for target in 0..6 {
            prop_assert_eq!(
                p.pending_bytes(target),
                Some(expected.pending[target]),
                "Mismatch at pending target {}", target
            );
            prop_assert_eq!(
                p.usage_percent(target),
                Some(expected.usage[target]),
                "Mismatch at usage target {}", target
            );
            prop_assert_eq!(
                p.peak_usage_percent(target),
                Some(expected.peak_usage[target]),
                "Mismatch at peak_usage target {}", target
            );
        }
    }
}

#[cfg(feature = "ubx_proto14")]
proptest! {
    #[test]
    fn test_parser_proto14_with_generated_mon_rxbuf_frames(
        (expected, frame) in ubx_mon_rxbuf_frame_strategy()
    ) {
        use ublox::proto14::{Proto14, PacketRef};

        let mut parser = ParserBuilder::new().with_protocol::<Proto14>().with_fixed_buffer::<1024>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto14(PacketRef::MonRxbuf(p)))) = it.next() else {
            panic!("Parser failed to parse a MON-RXBUF valid packet");
        };

        for target in 0..6 {
            prop_assert_eq!(p.pending_bytes(target), Some(expected.pending[target]));
            prop_assert_eq!(p.usage_percent(target), Some(expected.usage[target]));
            prop_assert_eq!(p.peak_usage_percent(target), Some(expected.peak_usage[target]));
        }
    }
}
