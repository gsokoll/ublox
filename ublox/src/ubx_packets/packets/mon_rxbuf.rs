//! MON-RXBUF: Receiver Buffer Status
//!
//! Provides information about the receiver buffer status for each target.

#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

#[allow(unused_imports, reason = "It is only unused in some feature sets")]
use crate::FieldIter;
use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::ubx_packet_recv;

/// Receiver Buffer Status
///
/// Provides information about the receiver buffer status for each target.
/// The 6 targets are indexed 0-5.
#[ubx_packet_recv]
#[ubx(class = 0x0a, id = 0x07, fixed_payload_len = 24)]
struct MonRxbuf {
    /// Number of bytes pending in receiver buffer for each target (6 x U2 = 12 bytes)
    pending: [u8; 12],

    /// Maximum usage receiver buffer during last sysmon period for each target (%)
    usage: [u8; 6],

    /// Maximum usage receiver buffer for each target (%)
    peak_usage: [u8; 6],
}

impl<'d> MonRxbufRef<'d> {
    /// Get the number of bytes pending in receiver buffer for a specific target (0-5).
    pub fn pending_bytes(&self, target: usize) -> Option<u16> {
        if target > 5 {
            return None;
        }
        let data = self.pending();
        let offset = target * 2;
        Some(u16::from_le_bytes([data[offset], data[offset + 1]]))
    }

    /// Get the maximum usage receiver buffer during last sysmon period for a specific target (0-5).
    /// Returns value as percentage (0-100%).
    pub fn usage_percent(&self, target: usize) -> Option<u8> {
        if target > 5 {
            return None;
        }
        Some(self.usage()[target])
    }

    /// Get the maximum (peak) usage receiver buffer for a specific target (0-5).
    /// Returns value as percentage (0-100%).
    pub fn peak_usage_percent(&self, target: usize) -> Option<u8> {
        if target > 5 {
            return None;
        }
        Some(self.peak_usage()[target])
    }
}
