//! MON-RXR: Receiver Status Information
//!
//! This message is sent when the receiver changes from or to backup mode.

#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

#[allow(unused_imports, reason = "It is only unused in some feature sets")]
use crate::FieldIter;
use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::ubx_packet_recv;

/// Receiver Status Information
///
/// This message is sent when the receiver changes from or to backup mode.
/// The receiver ready message indicates whether the receiver is awake.
#[ubx_packet_recv]
#[ubx(class = 0x0a, id = 0x21, fixed_payload_len = 1)]
struct MonRxr {
    /// Receiver status flags
    /// Bit 0: awake - not in backup mode
    flags: u8,
}

impl<'a> MonRxrRef<'a> {
    /// Returns true if the receiver is awake (not in backup mode).
    pub fn is_awake(&self) -> bool {
        (self.flags() & 0x01) != 0
    }
}
