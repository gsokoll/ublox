//! A proptest generator for U-Blox MON-RXR messages.
//!
//! This module provides a `proptest` strategy to generate byte-level
//! UBX frames containing a MON-RXR message.

use byteorder::{LittleEndian, WriteBytesExt};
use proptest::prelude::*;
use ublox::{ParserBuilder, UbxPacket};

/// Represents the payload of a UBX-MON-RXR message.
#[derive(Debug, Clone)]
pub struct MonRxr {
    /// Receiver status flags (bit 0: awake)
    pub flags: u8,
}

impl MonRxr {
    fn to_bytes(&self) -> Vec<u8> {
        vec![self.flags]
    }

    /// Returns true if the awake bit is set.
    pub fn is_awake(&self) -> bool {
        (self.flags & 0x01) != 0
    }
}

/// Strategy for generating a MonRxr payload.
fn mon_rxr_payload_strategy() -> impl Strategy<Value = MonRxr> {
    // Only bit 0 is defined, but we test all possible values
    any::<u8>().prop_map(|flags| MonRxr { flags })
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

/// Strategy that generates a complete, valid UBX frame containing a MON-RXR message.
pub fn ubx_mon_rxr_frame_strategy() -> impl Strategy<Value = (MonRxr, Vec<u8>)> {
    mon_rxr_payload_strategy().prop_map(|rxr| {
        let payload = rxr.to_bytes();
        let class_id = 0x0a;
        let message_id = 0x21;
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

        (rxr, final_frame)
    })
}

#[cfg(feature = "ubx_proto27")]
proptest! {
    #[test]
    fn test_parser_proto27_with_generated_mon_rxr_frames(
        (expected, frame) in ubx_mon_rxr_frame_strategy()
    ) {
        use ublox::proto27::{Proto27, PacketRef};

        let mut parser = ParserBuilder::new().with_protocol::<Proto27>().with_fixed_buffer::<1024>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto27(PacketRef::MonRxr(p)))) = it.next() else {
            panic!("Parser failed to parse a MON-RXR valid packet");
        };

        prop_assert_eq!(p.flags(), expected.flags);
        prop_assert_eq!(p.is_awake(), expected.is_awake());
    }
}

#[cfg(feature = "ubx_proto14")]
proptest! {
    #[test]
    fn test_parser_proto14_with_generated_mon_rxr_frames(
        (expected, frame) in ubx_mon_rxr_frame_strategy()
    ) {
        use ublox::proto14::{Proto14, PacketRef};

        let mut parser = ParserBuilder::new().with_protocol::<Proto14>().with_fixed_buffer::<1024>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto14(PacketRef::MonRxr(p)))) = it.next() else {
            panic!("Parser failed to parse a MON-RXR valid packet");
        };

        prop_assert_eq!(p.flags(), expected.flags);
        prop_assert_eq!(p.is_awake(), expected.is_awake());
    }
}
