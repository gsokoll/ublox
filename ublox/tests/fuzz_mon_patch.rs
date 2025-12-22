//! A proptest generator for U-Blox MON-PATCH messages.
//!
//! This module provides a `proptest` strategy to generate byte-level
//! UBX frames containing a MON-PATCH message with variable entries.

use byteorder::{LittleEndian, WriteBytesExt};
use proptest::prelude::*;
use ublox::{ParserBuilder, UbxPacket};

/// Represents a single patch entry (16 bytes).
#[derive(Debug, Clone)]
pub struct PatchEntry {
    pub activated: bool,
    pub location: u8, // 0-3
    pub comparator_number: u32,
    pub patch_address: u32,
    pub patch_data: u32,
}

impl PatchEntry {
    fn to_bytes(&self) -> Vec<u8> {
        let mut wtr = Vec::with_capacity(16);
        // patchInfo: bit 0 = activated, bits 2..1 = location
        let patch_info: u32 =
            (if self.activated { 1 } else { 0 }) | ((self.location as u32 & 0x03) << 1);
        wtr.write_u32::<LittleEndian>(patch_info).unwrap();
        wtr.write_u32::<LittleEndian>(self.comparator_number).unwrap();
        wtr.write_u32::<LittleEndian>(self.patch_address).unwrap();
        wtr.write_u32::<LittleEndian>(self.patch_data).unwrap();
        wtr
    }
}

/// Represents the payload of a UBX-MON-PATCH message.
#[derive(Debug, Clone)]
pub struct MonPatch {
    pub version: u16,
    pub entries: Vec<PatchEntry>,
}

impl MonPatch {
    fn to_bytes(&self) -> Vec<u8> {
        let mut wtr = Vec::with_capacity(4 + self.entries.len() * 16);
        wtr.write_u16::<LittleEndian>(self.version).unwrap();
        wtr.write_u16::<LittleEndian>(self.entries.len() as u16).unwrap();
        for entry in &self.entries {
            wtr.extend(entry.to_bytes());
        }
        wtr
    }
}

/// Strategy for generating a single patch entry.
fn patch_entry_strategy() -> impl Strategy<Value = PatchEntry> {
    (
        any::<bool>(),        // activated
        0u8..4u8,             // location (0-3)
        any::<u32>(),         // comparator_number
        any::<u32>(),         // patch_address
        any::<u32>(),         // patch_data
    )
        .prop_map(|(activated, location, comparator_number, patch_address, patch_data)| {
            PatchEntry {
                activated,
                location,
                comparator_number,
                patch_address,
                patch_data,
            }
        })
}

/// Strategy for generating a MonPatch payload with 0-8 entries.
fn mon_patch_payload_strategy() -> impl Strategy<Value = MonPatch> {
    (
        Just(0x0001u16), // version is always 0x0001
        prop::collection::vec(patch_entry_strategy(), 0..8),
    )
        .prop_map(|(version, entries)| MonPatch { version, entries })
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

/// Strategy that generates a complete, valid UBX frame containing a MON-PATCH message.
pub fn ubx_mon_patch_frame_strategy() -> impl Strategy<Value = (MonPatch, Vec<u8>)> {
    mon_patch_payload_strategy().prop_map(|patch| {
        let payload = patch.to_bytes();
        let class_id = 0x0a;
        let message_id = 0x27;
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

        (patch, final_frame)
    })
}

#[cfg(feature = "ubx_proto27")]
proptest! {
    #[test]
    fn test_parser_proto27_with_generated_mon_patch_frames(
        (expected, frame) in ubx_mon_patch_frame_strategy()
    ) {
        use ublox::proto27::{Proto27, PacketRef};

        let mut parser = ParserBuilder::new().with_protocol::<Proto27>().with_fixed_buffer::<1024>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto27(PacketRef::MonPatch(p)))) = it.next() else {
            panic!("Parser failed to parse a MON-PATCH valid packet");
        };

        // Verify header fields
        prop_assert_eq!(p.version(), expected.version);
        prop_assert_eq!(p.n_entries(), expected.entries.len() as u16);

        // Verify entries via iterator
        let parsed_entries: Vec<_> = p.entries().collect();
        prop_assert_eq!(parsed_entries.len(), expected.entries.len());

        for (i, (parsed, exp)) in parsed_entries.iter().zip(expected.entries.iter()).enumerate() {
            prop_assert_eq!(parsed.activated, exp.activated, "Entry {} activated mismatch", i);
            prop_assert_eq!(parsed.comparator_number, exp.comparator_number, "Entry {} comparator mismatch", i);
            prop_assert_eq!(parsed.patch_address, exp.patch_address, "Entry {} address mismatch", i);
            prop_assert_eq!(parsed.patch_data, exp.patch_data, "Entry {} data mismatch", i);
        }
    }
}

#[cfg(feature = "ubx_proto14")]
proptest! {
    #[test]
    fn test_parser_proto14_with_generated_mon_patch_frames(
        (expected, frame) in ubx_mon_patch_frame_strategy()
    ) {
        use ublox::proto14::{Proto14, PacketRef};

        let mut parser = ParserBuilder::new().with_protocol::<Proto14>().with_fixed_buffer::<1024>();
        let mut it = parser.consume_ubx(&frame);

        let Some(Ok(UbxPacket::Proto14(PacketRef::MonPatch(p)))) = it.next() else {
            panic!("Parser failed to parse a MON-PATCH valid packet");
        };

        prop_assert_eq!(p.version(), expected.version);
        prop_assert_eq!(p.n_entries(), expected.entries.len() as u16);

        let parsed_entries: Vec<_> = p.entries().collect();
        prop_assert_eq!(parsed_entries.len(), expected.entries.len());
    }
}
