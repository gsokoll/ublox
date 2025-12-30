//! Fuzz test for MON-TXBUF
//!
//! Auto-generated from ubx-protocol-schema

use byteorder::{LittleEndian, WriteBytesExt};
use proptest::prelude::*;
use ublox::{ParserBuilder, UbxPacket};

/// Expected values for MON-TXBUF
#[derive(Debug, Clone)]
pub struct ExpectedMonTxbuf {
    pub pending: [u16; 6],
    pub usage: [u8; 6],
    pub peak_usage: [u8; 6],
    pub t_usage: u8,
    pub t_peakusage: u8,
    pub errors: u8,
    pub reserved0: u8,
}

impl ExpectedMonTxbuf {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut wtr = Vec::new();
        for v in &self.pending { wtr.write_u16::<LittleEndian>(*v).unwrap(); }
        wtr.extend_from_slice(&self.usage);
        wtr.extend_from_slice(&self.peak_usage);
        wtr.push(self.t_usage);
        wtr.push(self.t_peakusage);
        wtr.push(self.errors);
        wtr.push(self.reserved0);
        wtr
    }
}

/// Proptest strategy for MonTxbuf
fn mon_txbuf_strategy() -> impl Strategy<Value = ExpectedMonTxbuf> {
    (
        prop::array::uniform6(any::<u16>()),
        prop::array::uniform6(any::<u8>()),
        prop::array::uniform6(any::<u8>()),
        any::<u8>(),
        any::<u8>(),
        any::<u8>(),
        any::<u8>()
    ).prop_map(|(
        pending, usage, peak_usage, t_usage, t_peakusage, errors, reserved0
    )| ExpectedMonTxbuf {
        pending,
        usage,
        peak_usage,
        t_usage,
        t_peakusage,
        errors,
        reserved0,
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

fn build_mon_txbuf_frame(expected: &ExpectedMonTxbuf) -> Vec<u8> {
    let payload = expected.to_bytes();
    let class_id: u8 = 0x0a;
    let msg_id: u8 = 0x08;
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

pub fn mon_txbuf_frame_strategy() -> impl Strategy<Value = (ExpectedMonTxbuf, Vec<u8>)> {
    mon_txbuf_strategy().prop_map(|expected| {
        let frame = build_mon_txbuf_frame(&expected);
        (expected, frame)
    })
}

proptest! {
    #[test]
    fn test_mon_txbuf_roundtrip(
        (expected, frame) in mon_txbuf_frame_strategy()
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
