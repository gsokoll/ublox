//! Fuzz test for MON-RXBUF
//!
//! Auto-generated from ubx-protocol-schema

use byteorder::{LittleEndian, WriteBytesExt};
use proptest::prelude::*;
use ublox::{ParserBuilder, UbxPacket};

/// Expected values for MON-RXBUF
#[derive(Debug, Clone)]
pub struct ExpectedMonRxbuf {
    pub pending: [u16; 6],
    pub usage: [u8; 6],
    pub peak_usage: [u8; 6],
}

impl ExpectedMonRxbuf {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut wtr = Vec::new();
        for v in &self.pending { wtr.write_u16::<LittleEndian>(*v).unwrap(); }
        wtr.extend_from_slice(&self.usage);
        wtr.extend_from_slice(&self.peak_usage);
        wtr
    }
}

/// Proptest strategy for MonRxbuf
fn mon_rxbuf_strategy() -> impl Strategy<Value = ExpectedMonRxbuf> {
    (
        prop::array::uniform6(any::<u16>()),
        prop::array::uniform6(any::<u8>()),
        prop::array::uniform6(any::<u8>())
    ).prop_map(|(
        pending, usage, peak_usage
    )| ExpectedMonRxbuf {
        pending,
        usage,
        peak_usage,
    })
}

fn calculate_checksum(data: &[u8]) -> (u8, u8) {
    let mut ck_a: u8 = 0;
    let mut ck_b: u8 = 0;
    for byte in data {
        ck_a = ck_a.wrapping_add(*byte);
        ck_b = ck_b.wrapping_add(ck_a);
    }
    (ck_a, ck_b)
}

fn build_mon_rxbuf_frame(expected: &ExpectedMonRxbuf) -> Vec<u8> {
    let payload = expected.to_bytes();
    let class_id: u8 = 0x0a;
    let msg_id: u8 = 0x07;
    let length = payload.len() as u16;

    let mut frame_core = Vec::with_capacity(4 + payload.len());
    frame_core.push(class_id);
    frame_core.push(msg_id);
    frame_core.write_u16::<LittleEndian>(length).unwrap();
    frame_core.extend_from_slice(&payload);

    let (ck_a, ck_b) = calculate_checksum(&frame_core);

    let mut frame = Vec::with_capacity(8 + payload.len());
    frame.push(0xB5);
    frame.push(0x62);
    frame.extend_from_slice(&frame_core);
    frame.push(ck_a);
    frame.push(ck_b);
    frame
}

pub fn mon_rxbuf_frame_strategy() -> impl Strategy<Value = (ExpectedMonRxbuf, Vec<u8>)> {
    mon_rxbuf_strategy().prop_map(|expected| {
        let frame = build_mon_rxbuf_frame(&expected);
        (expected, frame)
    })
}

proptest! {
    #[test]
    fn test_mon_rxbuf_roundtrip(
        (expected, frame) in mon_rxbuf_frame_strategy()
    ) {
        // Parse the generated frame
        let mut parser = ParserBuilder::default().build();
        let mut it = parser.consume_ubx(&frame);

        match it.next() {
            Some(Ok(packet)) => {
                // Frame parsed successfully
                // Add field-level assertions here based on packet type
            }
            Some(Err(e)) => prop_assert!(false, "Parse error: {:?}", e),
            None => prop_assert!(false, "No packet parsed"),
        }
    }
}
