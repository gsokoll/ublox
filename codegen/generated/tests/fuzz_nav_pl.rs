//! Fuzz test for NAV-PL
//!
//! Auto-generated from ubx-protocol-schema

use byteorder::{LittleEndian, WriteBytesExt};
use proptest::prelude::*;
use ublox::{ParserBuilder, UbxPacket};

/// Expected values for NAV-PL
#[derive(Debug, Clone)]
pub struct ExpectedNavPl {
    pub msg_version: u8,
    pub tmir_coeff: u8,
    pub tmir_exp: i8,
    pub pl_pos_valid: u8,
    pub pl_pos_frame: u8,
    pub pl_vel_valid: u8,
    pub pl_vel_frame: u8,
    pub pl_time_valid: u8,
    pub pl_pos_invalidity_reason: u8,
    pub pl_vel_invalidity_reason: u8,
    pub pl_time_invalidity_reason: u8,
    pub reserved0: u8,
    pub i_tow: u32,
    pub pl_pos1: u32,
    pub pl_pos2: u32,
    pub pl_pos3: u32,
    pub pl_vel1: u32,
    pub pl_vel2: u32,
    pub pl_vel3: u32,
    pub pl_pos_horiz_orient: u16,
    pub pl_vel_horiz_orient: u16,
    pub pl_time: u32,
    pub reserved1: [u8; 4],
}

impl ExpectedNavPl {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut wtr = Vec::new();
        wtr.push(self.msg_version);
        wtr.push(self.tmir_coeff);
        wtr.push(self.tmir_exp as u8);
        wtr.push(self.pl_pos_valid);
        wtr.push(self.pl_pos_frame);
        wtr.push(self.pl_vel_valid);
        wtr.push(self.pl_vel_frame);
        wtr.push(self.pl_time_valid);
        wtr.push(self.pl_pos_invalidity_reason);
        wtr.push(self.pl_vel_invalidity_reason);
        wtr.push(self.pl_time_invalidity_reason);
        wtr.push(self.reserved0);
        wtr.write_u32::<LittleEndian>(self.i_tow).unwrap();
        wtr.write_u32::<LittleEndian>(self.pl_pos1).unwrap();
        wtr.write_u32::<LittleEndian>(self.pl_pos2).unwrap();
        wtr.write_u32::<LittleEndian>(self.pl_pos3).unwrap();
        wtr.write_u32::<LittleEndian>(self.pl_vel1).unwrap();
        wtr.write_u32::<LittleEndian>(self.pl_vel2).unwrap();
        wtr.write_u32::<LittleEndian>(self.pl_vel3).unwrap();
        wtr.write_u16::<LittleEndian>(self.pl_pos_horiz_orient).unwrap();
        wtr.write_u16::<LittleEndian>(self.pl_vel_horiz_orient).unwrap();
        wtr.write_u32::<LittleEndian>(self.pl_time).unwrap();
        wtr.extend_from_slice(&self.reserved1);
        wtr
    }
}

/// Proptest strategy for NavPl
fn nav_pl_strategy() -> impl Strategy<Value = ExpectedNavPl> {
    (
        // Group 1
        (
            any::<u8>(),
            any::<u8>(),
            any::<i8>(),
            any::<u8>(),
            any::<u8>(),
            any::<u8>(),
            any::<u8>(),
            any::<u8>(),
            any::<u8>(),
            any::<u8>()
        ),
        // Group 2
        (
            any::<u8>(),
            any::<u8>(),
            any::<u32>(),
            any::<u32>(),
            any::<u32>(),
            any::<u32>(),
            any::<u32>(),
            any::<u32>(),
            any::<u32>(),
            any::<u16>()
        ),
        // Group 3
        (
            any::<u16>(),
            any::<u32>(),
            prop::array::uniform4(any::<u8>())
        )
    ).prop_map(|(
        (msg_version, tmir_coeff, tmir_exp, pl_pos_valid, pl_pos_frame, pl_vel_valid, pl_vel_frame, pl_time_valid, pl_pos_invalidity_reason, pl_vel_invalidity_reason),
        (pl_time_invalidity_reason, reserved0, i_tow, pl_pos1, pl_pos2, pl_pos3, pl_vel1, pl_vel2, pl_vel3, pl_pos_horiz_orient),
        (pl_vel_horiz_orient, pl_time, reserved1)
    )| ExpectedNavPl {
        msg_version,
        tmir_coeff,
        tmir_exp,
        pl_pos_valid,
        pl_pos_frame,
        pl_vel_valid,
        pl_vel_frame,
        pl_time_valid,
        pl_pos_invalidity_reason,
        pl_vel_invalidity_reason,
        pl_time_invalidity_reason,
        reserved0,
        i_tow,
        pl_pos1,
        pl_pos2,
        pl_pos3,
        pl_vel1,
        pl_vel2,
        pl_vel3,
        pl_pos_horiz_orient,
        pl_vel_horiz_orient,
        pl_time,
        reserved1,
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

fn build_nav_pl_frame(expected: &ExpectedNavPl) -> Vec<u8> {
    let payload = expected.to_bytes();
    let class_id: u8 = 0x01;
    let msg_id: u8 = 0x62;
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

pub fn nav_pl_frame_strategy() -> impl Strategy<Value = (ExpectedNavPl, Vec<u8>)> {
    nav_pl_strategy().prop_map(|expected| {
        let frame = build_nav_pl_frame(&expected);
        (expected, frame)
    })
}

proptest! {
    #[test]
    fn test_nav_pl_roundtrip(
        (expected, frame) in nav_pl_frame_strategy()
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
