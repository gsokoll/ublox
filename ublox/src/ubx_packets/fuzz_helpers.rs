//! Helper functions for fuzz testing UBX packets.
//!
//! This module provides utilities for building valid UBX frames
//! from payload data, primarily used by auto-generated fuzz strategies.

/// Calculate the UBX checksum for a message.
///
/// The checksum is calculated over class, id, length, and payload bytes
/// using the 8-bit Fletcher algorithm.
pub fn calculate_checksum(class: u8, id: u8, payload: &[u8]) -> (u8, u8) {
    let mut ck_a: u8 = 0;
    let mut ck_b: u8 = 0;

    let len = payload.len() as u16;
    let len_bytes = len.to_le_bytes();

    // Checksum covers: class, id, length (2 bytes), payload
    for &byte in [class, id, len_bytes[0], len_bytes[1]]
        .iter()
        .chain(payload.iter())
    {
        ck_a = ck_a.wrapping_add(byte);
        ck_b = ck_b.wrapping_add(ck_a);
    }

    (ck_a, ck_b)
}

/// Build a complete UBX frame from class, id, and payload.
///
/// Returns a `Vec<u8>` containing the full UBX frame:
/// - Sync bytes (0xB5, 0x62)
/// - Class byte
/// - ID byte  
/// - Length (2 bytes, little-endian)
/// - Payload
/// - Checksum (2 bytes)
pub fn build_ubx_frame(class: u8, id: u8, payload: &[u8]) -> Vec<u8> {
    let (ck_a, ck_b) = calculate_checksum(class, id, payload);
    let len = payload.len() as u16;
    let len_bytes = len.to_le_bytes();

    let mut frame = Vec::with_capacity(8 + payload.len());
    frame.push(0xB5); // Sync char 1
    frame.push(0x62); // Sync char 2
    frame.push(class);
    frame.push(id);
    frame.extend_from_slice(&len_bytes);
    frame.extend_from_slice(payload);
    frame.push(ck_a);
    frame.push(ck_b);

    frame
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum_empty_payload() {
        let (ck_a, ck_b) = calculate_checksum(0x01, 0x07, &[]);
        // class=0x01, id=0x07, len=0x00,0x00
        // ck_a = 0x01 + 0x07 + 0x00 + 0x00 = 0x08
        // ck_b = 0x01 + 0x08 + 0x08 + 0x08 = 0x19
        assert_eq!(ck_a, 0x08);
        assert_eq!(ck_b, 0x19);
    }

    #[test]
    fn test_build_frame_structure() {
        let payload = vec![0x01, 0x02, 0x03];
        let frame = build_ubx_frame(0x0A, 0x09, &payload);

        assert_eq!(frame[0], 0xB5); // Sync 1
        assert_eq!(frame[1], 0x62); // Sync 2
        assert_eq!(frame[2], 0x0A); // Class
        assert_eq!(frame[3], 0x09); // ID
        assert_eq!(frame[4], 0x03); // Length low byte
        assert_eq!(frame[5], 0x00); // Length high byte
        assert_eq!(&frame[6..9], &[0x01, 0x02, 0x03]); // Payload
        // Last two bytes are checksum
        assert_eq!(frame.len(), 11);
    }
}
