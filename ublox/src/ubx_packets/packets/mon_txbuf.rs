//! MON-TXBUF: Transmitter Buffer Status
//!
//! Provides information about the transmitter buffer status for each target.

#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

#[allow(unused_imports, reason = "It is only unused in some feature sets")]
use crate::FieldIter;
use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::ubx_packet_recv;

/// Transmitter Buffer Status
///
/// Provides information about the transmitter buffer status for each target.
/// The 6 targets are indexed 0-5.
#[ubx_packet_recv]
#[ubx(class = 0x0a, id = 0x08, fixed_payload_len = 28)]
struct MonTxbuf {
    /// Number of bytes pending in transmitter buffer for each target (6 x U2 = 12 bytes)
    pending: [u8; 12],

    /// Maximum usage transmitter buffer during last sysmon period for each target (%)
    usage: [u8; 6],

    /// Maximum usage transmitter buffer for each target (%)
    peak_usage: [u8; 6],

    /// Maximum usage of transmitter buffer during last sysmon period for all targets (%)
    t_usage: u8,

    /// Maximum usage of transmitter buffer for all targets (%)
    t_peak_usage: u8,

    /// Error bitmask
    /// Bits 5..0: limit - buffer limit of corresponding target reached
    /// Bit 6: mem - memory allocation error
    /// Bit 7: alloc - allocation error (TX buffer full)
    errors: u8,

    /// Reserved
    reserved0: u8,
}

impl<'d> MonTxbufRef<'d> {
    /// Get the number of bytes pending in transmitter buffer for a specific target (0-5).
    pub fn pending_bytes(&self, target: usize) -> Option<u16> {
        if target > 5 {
            return None;
        }
        let data = self.pending();
        let offset = target * 2;
        Some(u16::from_le_bytes([data[offset], data[offset + 1]]))
    }

    /// Get the maximum usage transmitter buffer during last sysmon period for a specific target (0-5).
    /// Returns value as percentage (0-100%).
    pub fn usage_percent(&self, target: usize) -> Option<u8> {
        if target > 5 {
            return None;
        }
        Some(self.usage()[target])
    }

    /// Get the maximum (peak) usage transmitter buffer for a specific target (0-5).
    /// Returns value as percentage (0-100%).
    pub fn peak_usage_percent(&self, target: usize) -> Option<u8> {
        if target > 5 {
            return None;
        }
        Some(self.peak_usage()[target])
    }

    /// Check if buffer limit was reached for a specific target (0-5).
    pub fn limit_reached(&self, target: usize) -> bool {
        if target > 5 {
            return false;
        }
        (self.errors() & (1 << target)) != 0
    }

    /// Check if memory allocation error occurred.
    pub fn mem_error(&self) -> bool {
        (self.errors() & 0x40) != 0
    }

    /// Check if allocation error occurred (TX buffer full).
    pub fn alloc_error(&self) -> bool {
        (self.errors() & 0x80) != 0
    }
}
