//! A proptest generator for U-Blox MON-TXBUF messages.
//!
//! This module provides a `proptest` strategy to generate byte-level
//! UBX frames containing a MON-TXBUF message.

use byteorder::{LittleEndian, WriteBytesExt};
use proptest::prelude::*;
use ublox::{ParserBuilder, UbxPacket};

/// Represents the payload of a UBX-MON-TXBUF message.
#[derive(Debug, Clone)]
pub struct MonTxbuf {
    /// Number of bytes pending in transmitter buffer for each target (6 targets)
    pub pending: [u16; 6],
    /// Maximum usage transmitter buffer during last sysmon period for each target (%)
    pub usage: [u8; 6],
    /// Maximum usage transmitter buffer for each target (%)
    pub peak_usage: [u8; 6],
    /// Maximum usage of transmitter buffer during last sysmon period for all targets (%)
    pub t_usage: u8,
    /// Maximum usage of transmitter buffer for all targets (%)
    pub t_peak_usage: u8,
    /// Error bitmask
    pub errors: u8,
}

impl MonTxbuf {
    fn to_bytes(&self) -> Vec<u8> {
        let mut wtr = Vec::with_capacity(28);
        for p in &self.pending {
            wtr.write_u16::<LittleEndian>(*p).unwrap();
        }
        wtr.extend_from_slice(&self.usage);
        wtr.extend_from_slice(&self.peak_usage);
        wtr.push(self.t_usage);
        wtr.push(self.t_peak_usage);
        wtr.push(self.errors);
        wtr.push(0); // reserved0
        wtr
    }

    /// Check if buffer limit was reached for a specific target (0-5).
    pub fn limit_reached(&self, target: usize) -> bool {
        if target > 5 {
            return false;
        }
        (self.errors & (1 << target)) != 0
    }

    /// Check if memory allocation error occurred.
    pub fn mem_error(&self) -> bool {
        (self.errors & 0x40) != 0
    }

    /// Check if allocation error occurred (TX buffer full).
    pub fn alloc_error(&self) -> bool {
        (self.errors & 0x80) != 0
    }
}

/// Strategy for generating a MonTxbuf payload.
fn mon_txbuf_payload_strategy() -> impl Strategy<Value = MonTxbuf> {
    (
        prop::array::uniform6(any::<u16>()),
        prop::array::uniform6(0u8..=100u8), // usage is percentage
        prop::array::uniform6(0u8..=100u8), // peak_usage is percentage
        0u8..=100u8,                         // t_usage
        0u8..=100u8,                         // t_peak_usage
        any::<u8>(),                         // errors
    )
        .prop_map(|(pending, usage, peak_usage, t_usage, t_peak_usage, errors)| MonTxbuf {
            pending,
            usage,
            peak_usage,
            t_usage,
            t_peak_usage,
            errors,
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

/// Strategy that generates a complete, valid UBX frame containing a MON-TXBUF message.
pub fn ubx_mon_txbuf_frame_strategy() -> impl Strategy<Value = (MonTxbuf, Vec<u8>)> {
    mon_txbuf_payload_strategy().prop_map(|txbuf| {
        let payload = txbuf.to_bytes();
        let class_id = 0x0a;
        let message_id = 0x08;
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

        (txbuf, final_frame)
    })
}

#[cfg(feature = "ubx_proto27")]
proptest! {
    #[test]
    fn test_parser_proto27_with_generated_mon_txbuf_frames(
        (expected, frame) in ubx_mon_txbuf_frame_strategy()
    ) {
        use ublox::proto27::{Proto27, PacketRef};

        let mut parser = ParserBuilder::new().with_protocol::<Proto27>().with_fixed_buffer::<1024>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto27(PacketRef::MonTxbuf(p)))) = it.next() else {
            panic!("Parser failed to parse a MON-TXBUF valid packet");
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
            prop_assert_eq!(
                p.limit_reached(target),
                expected.limit_reached(target),
                "Mismatch at limit_reached target {}", target
            );
        }

        prop_assert_eq!(p.t_usage(), expected.t_usage);
        prop_assert_eq!(p.t_peak_usage(), expected.t_peak_usage);
        prop_assert_eq!(p.errors(), expected.errors);
        prop_assert_eq!(p.mem_error(), expected.mem_error());
        prop_assert_eq!(p.alloc_error(), expected.alloc_error());
    }
}

#[cfg(feature = "ubx_proto14")]
proptest! {
    #[test]
    fn test_parser_proto14_with_generated_mon_txbuf_frames(
        (expected, frame) in ubx_mon_txbuf_frame_strategy()
    ) {
        use ublox::proto14::{Proto14, PacketRef};

        let mut parser = ParserBuilder::new().with_protocol::<Proto14>().with_fixed_buffer::<1024>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto14(PacketRef::MonTxbuf(p)))) = it.next() else {
            panic!("Parser failed to parse a MON-TXBUF valid packet");
        };

        for target in 0..6 {
            prop_assert_eq!(p.pending_bytes(target), Some(expected.pending[target]));
            prop_assert_eq!(p.usage_percent(target), Some(expected.usage[target]));
            prop_assert_eq!(p.peak_usage_percent(target), Some(expected.peak_usage[target]));
        }
        prop_assert_eq!(p.t_usage(), expected.t_usage);
        prop_assert_eq!(p.t_peak_usage(), expected.t_peak_usage);
    }
}
