//! Auto-generated from ubx-protocol-schema
//!
//! MON-RXBUF message definition

use ublox_derive::{ubx_extend, ubx_packet_recv};

/// Receiver buffer status
#[ubx_packet_recv]
#[ubx(class = 0x0a, id = 0x07, fixed_payload_len = 24)]
struct MonRxbuf {
    /// Number of bytes pending in receiver buffer for each target
    pending: [u16; 6],
    /// Maximum usage receiver buffer during the last system period for each target
    usage: [u8; 6],
    /// Maximum usage receiver buffer for each target
    peak_usage: [u8; 6],
}
