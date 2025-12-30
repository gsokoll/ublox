//! Auto-generated from ubx-protocol-schema
//!
//! MON-SPAN message definition

use ublox_derive::{ubx_extend, ubx_packet_recv};

/// Signal characteristics - basic spectrum analyzer displaying one spectrum for each of the receiver's existing RF paths
#[ubx_packet_recv]
#[ubx(class = 0x0a, id = 0x31, fixed_payload_len = 4)]
struct MonSpan {
    /// Message version (0x00 for this version)
    version: u8,
    /// Number of RF blocks included
    num_rf_blocks: u8,
    /// Reserved
    reserved0: [u8; 2],
}
