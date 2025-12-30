//! Auto-generated from ubx-protocol-schema
//!
//! NAV-PL message definition

use ublox_derive::{ubx_extend, ubx_packet_recv};

/// Protection level information
#[ubx_packet_recv]
#[ubx(class = 0x01, id = 0x62, fixed_payload_len = 52)]
struct NavPl {
    /// Message version (0x01 for this version)
    msg_version: u8,
    /// Target misleading information risk (TMIR) [%MI/epoch], coefficient integer nu...
    tmir_coeff: u8,
    /// Target misleading information risk (TMIR) [%MI/epoch], exponent integer numbe...
    tmir_exp: i8,
    /// Position protection level validity
    pl_pos_valid: u8,
    /// Position protection level frame
    pl_pos_frame: u8,
    /// Velocity protection level validity
    pl_vel_valid: u8,
    /// Velocity protection level frame
    pl_vel_frame: u8,
    /// Time protection level validity
    pl_time_valid: u8,
    /// Position protection level invalidity reason
    pl_pos_invalidity_reason: u8,
    /// Velocity protection level invalidity reason
    pl_vel_invalidity_reason: u8,
    /// Time protection level invalidity reason
    pl_time_invalidity_reason: u8,
    /// Reserved
    reserved0: u8,
    /// GPS time of week
    i_tow: u32,
    /// First axis of position protection level value, given in coordinate frame of p...
    pl_pos1: u32,
    /// Second axis of position protection level value, given in coordinate frame of ...
    pl_pos2: u32,
    /// Third axis of position protection level value, given in coordinate frame of p...
    pl_pos3: u32,
    /// First axis of velocity protection level value, given in coordinate frame of p...
    pl_vel1: u32,
    /// Second axis of velocity protection level value, given in coordinate frame of ...
    pl_vel2: u32,
    /// Third axis of velocity protection level value, given in coordinate frame of p...
    pl_vel3: u32,
    /// Orientation of HorizSemiMajorAxis (see plPosFrame) of horizontal ellipse posi...
    pl_pos_horiz_orient: u16,
    /// Orientation of HorizSemiMajorAxis (see plVelFrame) of horizontal ellipse velo...
    pl_vel_horiz_orient: u16,
    /// Time protection level value, w.r.t. the given target misleading information r...
    pl_time: u32,
    /// Reserved
    reserved1: [u8; 4],
}
