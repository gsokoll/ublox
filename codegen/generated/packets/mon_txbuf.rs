//! Auto-generated from ubx-protocol-schema
//!
//! MON-TXBUF message definition

use ublox_derive::{ubx_extend, ubx_packet_recv};

/// Transmitter buffer status
#[ubx_packet_recv]
#[ubx(class = 0x0a, id = 0x08, fixed_payload_len = 28)]
struct MonTxbuf {
    /// Number of bytes pending in transmitter buffer for each target
    pending: [u16; 6],
    /// Maximum usage transmitter buffer during the last sysmon period for each target
    usage: [u8; 6],
    /// Maximum usage transmitter buffer for each target
    peak_usage: [u8; 6],
    /// Maximum usage of transmitter buffer during the last sysmon period for all tar...
    t_usage: u8,
    /// Maximum usage of transmitter buffer for all targets
    t_peakusage: u8,
    /// Error bitmask
    errors: u8,
    /// Reserved
    reserved0: u8,
}
