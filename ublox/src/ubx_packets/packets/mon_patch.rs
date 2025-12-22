//! MON-PATCH: Installed Patches Information
//!
//! Reports information about patches installed and currently enabled on the receiver.

#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

#[allow(unused_imports, reason = "It is only unused in some feature sets")]
use crate::FieldIter;
use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::ubx_packet_recv;

/// Installed Patches Information
///
/// Reports information about patches installed and currently enabled on the receiver.
/// An enabled patch is considered active when the receiver executes from the code
/// space where the patch resides on.
#[ubx_packet_recv]
#[ubx(class = 0x0a, id = 0x27, max_payload_len = 260)] // 4 + 16 * 16 entries max
struct MonPatch {
    /// Message version (0x0001 for this version)
    version: u16,

    /// Total number of reported patches
    n_entries: u16,

    /// Patch entry blocks (repeated n_entries times, 16 bytes each)
    #[ubx(map_type = MonPatchEntryIter, may_fail,
          from = MonPatchEntryIter::new,
          is_valid = MonPatchEntryIter::is_valid)]
    entries: [u8; 0],
}

/// Patch storage location
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum PatchLocation {
    /// One-Time Programmable memory
    Otp,
    /// ROM
    Rom,
    /// Battery-Backed RAM
    Bbr,
    /// File system
    FileSystem,
}

impl From<u8> for PatchLocation {
    fn from(value: u8) -> Self {
        match value & 0x03 {
            0 => PatchLocation::Otp,
            1 => PatchLocation::Rom,
            2 => PatchLocation::Bbr,
            3 => PatchLocation::FileSystem,
            _ => unreachable!(),
        }
    }
}

/// Information for a single patch entry
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MonPatchEntry {
    /// Whether the patch is active (1) or not (0)
    pub activated: bool,
    /// Where the patch is stored
    pub location: PatchLocation,
    /// The number of the comparator
    pub comparator_number: u32,
    /// The address that is targeted by the patch
    pub patch_address: u32,
    /// The data that is inserted at the patch address
    pub patch_data: u32,
}

/// Iterator for MON-PATCH entry blocks
#[derive(Debug, Clone)]
pub struct MonPatchEntryIter<'d> {
    data: &'d [u8],
    offset: usize,
}

impl<'d> MonPatchEntryIter<'d> {
    /// Construct iterator from raw patch entry payload bytes.
    fn new(data: &'d [u8]) -> Self {
        Self { data, offset: 0 }
    }

    /// Validate raw repeated-group payload: must be a multiple of 16 bytes.
    #[allow(
        dead_code,
        reason = "Used by ubx_packet_recv macro for validation"
    )]
    fn is_valid(payload: &[u8]) -> bool {
        payload.len() % 16 == 0
    }
}

impl core::iter::Iterator for MonPatchEntryIter<'_> {
    type Item = MonPatchEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let chunk = self.data.get(self.offset..self.offset + 16)?;

        let patch_info = u32::from_le_bytes(chunk[0..4].try_into().ok()?);
        let activated = (patch_info & 0x01) != 0;
        let location = PatchLocation::from(((patch_info >> 1) & 0x03) as u8);

        let entry = MonPatchEntry {
            activated,
            location,
            comparator_number: u32::from_le_bytes(chunk[4..8].try_into().ok()?),
            patch_address: u32::from_le_bytes(chunk[8..12].try_into().ok()?),
            patch_data: u32::from_le_bytes(chunk[12..16].try_into().ok()?),
        };

        self.offset += 16;
        Some(entry)
    }
}
