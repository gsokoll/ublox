//! MON-MSGPP: Message Parse and Process Status
//!
//! Provides message parse and process counts per port and protocol.
//! This message is deprecated; use UBX-MON-COMMS instead for newer receivers.

#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

#[allow(unused_imports, reason = "It is only unused in some feature sets")]
use crate::FieldIter;
use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::ubx_packet_recv;

/// Message Parse and Process Status
///
/// Provides counts of successfully parsed messages for each protocol on each port,
/// plus the number of skipped bytes per port.
///
/// Protocol indices: 0=UBX, 1=NMEA, 2=RTCM2, 3=RTCM3, 4-7=reserved
///
/// This message is deprecated. Use UBX-MON-COMMS instead for newer receivers.
#[ubx_packet_recv]
#[ubx(class = 0x0a, id = 0x06, fixed_payload_len = 120)]
struct MonMsgpp {
    /// Number of successfully parsed messages for each protocol on port 0
    /// (8 x U2 = 16 bytes)
    msg1: [u8; 16],
    /// Number of successfully parsed messages for each protocol on port 1
    msg2: [u8; 16],
    /// Number of successfully parsed messages for each protocol on port 2
    msg3: [u8; 16],
    /// Number of successfully parsed messages for each protocol on port 3
    msg4: [u8; 16],
    /// Number of successfully parsed messages for each protocol on port 4
    msg5: [u8; 16],
    /// Number of successfully parsed messages for each protocol on port 5
    msg6: [u8; 16],
    /// Number of skipped bytes for each port (6 x U4 = 24 bytes)
    skipped: [u8; 24],
}

impl<'d> MonMsgppRef<'d> {
    /// Get message counts for a specific port (0-5) and protocol (0-7).
    /// Protocol indices: 0=UBX, 1=NMEA, 2=RTCM2, 3=RTCM3, 4-7=reserved
    pub fn msg_count(&self, port: usize, protocol: usize) -> Option<u16> {
        if port > 5 || protocol > 7 {
            return None;
        }
        let data = match port {
            0 => self.msg1(),
            1 => self.msg2(),
            2 => self.msg3(),
            3 => self.msg4(),
            4 => self.msg5(),
            5 => self.msg6(),
            _ => return None,
        };
        let offset = protocol * 2;
        Some(u16::from_le_bytes([data[offset], data[offset + 1]]))
    }

    /// Get skipped bytes count for a specific port (0-5).
    pub fn skipped_bytes(&self, port: usize) -> Option<u32> {
        if port > 5 {
            return None;
        }
        let data = self.skipped();
        let offset = port * 4;
        Some(u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]))
    }
}
