#![cfg(feature = "ubx_proto40")]
//! Protocol 40 specific types
//!
//! Placeholder module for F10-generation (NEO-F10N/T) proto40-only items;
//! to be populated in subsequent PRs. The current `PacketRef` mirrors
//! [`crate::UbxUnknownPacketRef`] so that the protocol can be selected and
//! parsed against without any proto40-specific message types having been
//! added yet.

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use crate::{ParserError, UbxUnknownPacketRef};

/// All proto40 packets. Currently only [`PacketRef::Unknown`] is generated
/// because no proto40-specific message types have been defined yet.
#[derive(Debug)]
#[non_exhaustive]
pub enum PacketRef<'a> {
    Unknown(UbxUnknownPacketRef<'a>),
}

impl<'a> From<PacketRef<'a>> for crate::UbxPacket<'a> {
    fn from(packet: PacketRef<'a>) -> Self {
        crate::UbxPacket::Proto40(packet)
    }
}

/// Tag for protocol 40 packets
pub struct Proto40;

impl crate::UbxProtocol for Proto40 {
    type PacketRef<'a> = PacketRef<'a>;
    /// No proto40-specific message types defined yet, so the receive-side
    /// payload bound is zero until messages are added in a follow-up PR.
    const MAX_PAYLOAD_LEN: u16 = 0;

    fn match_packet(
        class_id: u8,
        msg_id: u8,
        payload: &[u8],
    ) -> Result<Self::PacketRef<'_>, ParserError> {
        Ok(PacketRef::Unknown(UbxUnknownPacketRef {
            payload,
            class: class_id,
            msg_id,
        }))
    }
}

#[cfg(any(feature = "std", feature = "alloc"))]
impl core::default::Default for crate::Parser<Vec<u8>, Proto40> {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}
