//! MON-SPAN: Signal Characteristics
//!
//! This message provides RF spectrum analyzer output for each of the receiver's
//! existing RF paths. It displays 256 bins with amplitude data for comparative
//! analysis rather than absolute and precise spectrum overview.

#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

#[allow(unused_imports, reason = "It is only unused in some feature sets")]
use crate::FieldIter;
use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::ubx_packet_recv;

/// Signal Characteristics - RF spectrum analyzer
///
/// This message provides spectrum data for each RF path with 256 bins
/// containing amplitude data. The center frequency at each bin can be
/// computed as: f(i) = center + span * (i - 127) / 256
#[ubx_packet_recv]
#[ubx(class = 0x0a, id = 0x31, max_payload_len = 1092)] // 4 + 4 * 272 max
struct MonSpan {
    /// Message version (0x00 for this version)
    version: u8,

    /// Number of RF blocks included
    num_rf_blocks: u8,

    /// Reserved
    reserved0: [u8; 2],

    /// RF block data (repeated num_rf_blocks times, 272 bytes each)
    #[ubx(map_type = MonSpanBlockIter, may_fail,
          from = MonSpanBlockIter::new,
          is_valid = MonSpanBlockIter::is_valid)]
    blocks: [u8; 0],
}

/// Information for a single RF spectrum block (272 bytes)
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MonSpanBlock {
    /// Spectrum data (256 bins with amplitude in dB, scale 2^-2)
    /// Value in dB = spectrum[i] * 0.25
    pub spectrum: [u8; 256],
    /// Spectrum span in Hz
    pub span: u32,
    /// Resolution of the spectrum in Hz
    pub res: u32,
    /// Center of spectrum span in Hz
    pub center: u32,
    /// Programmable gain amplifier in dB
    pub pga: u8,
}

impl MonSpanBlock {
    /// Get the amplitude at a specific bin index (0-255) in dB.
    /// The value is scaled by 2^-2 (multiply by 0.25).
    pub fn amplitude_db(&self, bin: usize) -> Option<f32> {
        if bin > 255 {
            return None;
        }
        Some(self.spectrum[bin] as f32 * 0.25)
    }

    /// Calculate the center frequency for a specific bin index (0-255) in Hz.
    /// f(i) = center + span * (i - 127) / 256
    pub fn bin_frequency_hz(&self, bin: usize) -> Option<f64> {
        if bin > 255 {
            return None;
        }
        let i = bin as i32;
        Some(self.center as f64 + (self.span as f64 * (i - 127) as f64) / 256.0)
    }
}

/// Iterator for MON-SPAN RF blocks
#[derive(Debug, Clone)]
pub struct MonSpanBlockIter<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> MonSpanBlockIter<'a> {
    /// Construct iterator from raw RF block payload bytes.
    fn new(data: &'a [u8]) -> Self {
        Self { data, offset: 0 }
    }

    /// Validate raw repeated-group payload: must be a multiple of 272 bytes.
    #[allow(
        dead_code,
        reason = "Used by ubx_packet_recv macro for validation"
    )]
    fn is_valid(payload: &[u8]) -> bool {
        payload.len() % 272 == 0
    }
}

impl core::iter::Iterator for MonSpanBlockIter<'_> {
    type Item = MonSpanBlock;

    fn next(&mut self) -> Option<Self::Item> {
        let chunk = self.data.get(self.offset..self.offset + 272)?;

        let mut spectrum = [0u8; 256];
        spectrum.copy_from_slice(&chunk[0..256]);

        let block = MonSpanBlock {
            spectrum,
            span: u32::from_le_bytes(chunk[256..260].try_into().ok()?),
            res: u32::from_le_bytes(chunk[260..264].try_into().ok()?),
            center: u32::from_le_bytes(chunk[264..268].try_into().ok()?),
            pga: chunk[268],
            // bytes 269..272 are reserved
        };

        self.offset += 272;
        Some(block)
    }
}
