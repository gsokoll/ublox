//! A proptest generator for U-Blox MON-MSGPP messages.
//!
//! This module provides a `proptest` strategy to generate byte-level
//! UBX frames containing a MON-MSGPP message.

use byteorder::{LittleEndian, WriteBytesExt};
use proptest::prelude::*;
use ublox::{ParserBuilder, UbxPacket};

/// UBX Sync Character 1 (0xB5 = 'µ')
const SYNC_CHAR_1: u8 = 0xB5;
/// UBX Sync Character 2 (0x62 = 'b')
const SYNC_CHAR_2: u8 = 0x62;

/// Represents the payload of a UBX-MON-MSGPP message (120 bytes fixed).
#[derive(Debug, Clone)]
pub struct MonMsgpp {
    /// Message counts per protocol for each port (6 ports x 8 protocols)
    pub msg: [[u16; 8]; 6],
    /// Skipped bytes per port
    pub skipped: [u32; 6],
}

impl MonMsgpp {
    /// Serializes the payload into bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut wtr = Vec::with_capacity(120);
        // Write msg1-msg6 (6 ports x 8 protocols x 2 bytes = 96 bytes)
        for port in 0..6 {
            for protocol in 0..8 {
                wtr.write_u16::<LittleEndian>(self.msg[port][protocol]).unwrap();
            }
        }
        // Write skipped (6 ports x 4 bytes = 24 bytes)
        for port in 0..6 {
            wtr.write_u32::<LittleEndian>(self.skipped[port]).unwrap();
        }
        wtr
    }
}

/// A proptest strategy for generating a MonMsgpp payload.
fn mon_msgpp_payload_strategy() -> impl Strategy<Value = MonMsgpp> {
    (
        // Generate msg arrays for 6 ports
        prop::array::uniform6(prop::array::uniform8(any::<u16>())),
        // Generate skipped array for 6 ports
        prop::array::uniform6(any::<u32>()),
    )
        .prop_map(|(msg, skipped)| MonMsgpp { msg, skipped })
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

/// A proptest strategy that generates a complete, valid UBX frame
/// containing a MON-MSGPP message.
pub fn ubx_mon_msgpp_frame_strategy() -> impl Strategy<Value = (MonMsgpp, Vec<u8>)> {
    mon_msgpp_payload_strategy().prop_map(|msgpp| {
        let payload = msgpp.to_bytes();
        let class_id = 0x0a;
        let message_id = 0x06;
        let length = payload.len() as u16;

        let mut frame_core = Vec::with_capacity(4 + payload.len());
        frame_core.push(class_id);
        frame_core.push(message_id);
        frame_core.write_u16::<LittleEndian>(length).unwrap();
        frame_core.extend_from_slice(&payload);

        let (ck_a, ck_b) = calculate_checksum(&frame_core);

        let mut final_frame = Vec::with_capacity(8 + payload.len());
        final_frame.push(SYNC_CHAR_1);
        final_frame.push(SYNC_CHAR_2);
        final_frame.extend_from_slice(&frame_core);
        final_frame.push(ck_a);
        final_frame.push(ck_b);

        (msgpp, final_frame)
    })
}

#[cfg(feature = "ubx_proto27")]
proptest! {
    #[test]
    fn test_parser_proto27_with_generated_mon_msgpp_frames(
        (expected, frame) in ubx_mon_msgpp_frame_strategy()
    ) {
        use ublox::proto27::{Proto27, PacketRef};

        let mut parser = ParserBuilder::new().with_protocol::<Proto27>().with_fixed_buffer::<1024>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto27(PacketRef::MonMsgpp(p)))) = it.next() else {
            panic!("Parser failed to parse a MON-MSGPP valid packet");
        };

        // Verify parsed values match expected using accessor methods
        for port in 0..6 {
            for protocol in 0..8 {
                prop_assert_eq!(
                    p.msg_count(port, protocol),
                    Some(expected.msg[port][protocol]),
                    "Mismatch at port {} protocol {}", port, protocol
                );
            }
            prop_assert_eq!(
                p.skipped_bytes(port),
                Some(expected.skipped[port]),
                "Mismatch at skipped port {}", port
            );
        }
    }
}

#[cfg(feature = "ubx_proto14")]
proptest! {
    #[test]
    fn test_parser_proto14_with_generated_mon_msgpp_frames(
        (expected, frame) in ubx_mon_msgpp_frame_strategy()
    ) {
        use ublox::proto14::{Proto14, PacketRef};

        let mut parser = ParserBuilder::new().with_protocol::<Proto14>().with_fixed_buffer::<1024>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto14(PacketRef::MonMsgpp(p)))) = it.next() else {
            panic!("Parser failed to parse a MON-MSGPP valid packet");
        };

        // Verify parsed values match expected
        for port in 0..6 {
            for protocol in 0..8 {
                prop_assert_eq!(
                    p.msg_count(port, protocol),
                    Some(expected.msg[port][protocol])
                );
            }
            prop_assert_eq!(p.skipped_bytes(port), Some(expected.skipped[port]));
        }
    }
}
