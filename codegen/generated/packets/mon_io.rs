//! Auto-generated from ubx-protocol-schema
//!
//! MON-IO message definition

use ublox_derive::{ubx_extend, ubx_packet_recv};

/// I/O system status
#[ubx_packet_recv]
#[ubx(class = 0x0a, id = 0x02, fixed_payload_len = 0)]
struct MonIo {
}
