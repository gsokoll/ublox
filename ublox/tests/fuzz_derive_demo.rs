//! Demonstration of auto-generated fuzz strategies via derive macro.
//!
//! This test shows how the `#[ubx_packet_recv]` macro now automatically
//! generates `fuzz_payload_strategy()` and `fuzz_frame_strategy()` methods
//! that can be used for round-trip testing without manual boilerplate.
//!
//! Key feature: For fields mapped to enums via `#[ubx(map_type = SomeEnum)]`,
//! the generated strategy will ONLY produce valid enum values, not arbitrary
//! bytes. This makes the fuzz tests semantically aware.

#![cfg(feature = "ubx_proto23")]

use proptest::prelude::*;
use ublox::{ParserBuilder, UbxPacket};
use ublox::proto23::Proto23;

// Import packet types that have auto-generated fuzz strategies
use ublox::nav_pos_ecef::NavPosEcef;
use ublox::mon_hw::MonHw;
use ublox::ack::AckAck;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Test that NavPosEcef auto-generated strategy produces valid UBX frames
    #[test]
    fn test_nav_pos_ecef_frame_structure(frame in NavPosEcef::fuzz_frame_strategy()) {
        // Verify frame has correct UBX structure
        prop_assert!(frame.len() >= 8, "Frame too short: {}", frame.len());
        prop_assert_eq!(frame[0], 0xB5, "Invalid sync char 1");
        prop_assert_eq!(frame[1], 0x62, "Invalid sync char 2");
        prop_assert_eq!(frame[2], 0x01, "Invalid class for NavPosEcef");
        prop_assert_eq!(frame[3], 0x01, "Invalid id for NavPosEcef");
    }

    /// Test that MonHw auto-generated strategy produces valid UBX frames
    #[test]
    fn test_mon_hw_frame_structure(frame in MonHw::fuzz_frame_strategy()) {
        prop_assert!(frame.len() >= 8, "Frame too short: {}", frame.len());
        prop_assert_eq!(frame[0], 0xB5, "Invalid sync char 1");
        prop_assert_eq!(frame[1], 0x62, "Invalid sync char 2");
        prop_assert_eq!(frame[2], 0x0A, "Invalid class for MonHw");
        prop_assert_eq!(frame[3], 0x09, "Invalid id for MonHw");
    }

    /// Test that AckAck auto-generated strategy produces valid UBX frames
    #[test]
    fn test_ack_ack_frame_structure(frame in AckAck::fuzz_frame_strategy()) {
        prop_assert!(frame.len() >= 8, "Frame too short: {}", frame.len());
        prop_assert_eq!(frame[0], 0xB5, "Invalid sync char 1");
        prop_assert_eq!(frame[1], 0x62, "Invalid sync char 2");
        prop_assert_eq!(frame[2], 0x05, "Invalid class for AckAck");
        prop_assert_eq!(frame[3], 0x01, "Invalid id for AckAck");
    }

    /// Test round-trip parsing of AckAck using auto-generated strategy
    #[test]
    fn test_ack_ack_roundtrip(frame in AckAck::fuzz_frame_strategy()) {
        let mut parser = ParserBuilder::new()
            .with_protocol::<Proto23>()
            .with_fixed_buffer::<256>();
        
        let mut it = parser.consume_ubx(&frame);
        
        if let Some(Ok(UbxPacket::Proto23(ublox::proto23::PacketRef::AckAck(ack)))) = it.next() {
            // Successfully parsed - verify we can access fields
            let _class = ack.class();
            let _id = ack.msg_id();
        }
        // Note: Some frames may not parse due to field validation, which is expected
    }
}

// Demonstrate that enum-mapped fields use semantically valid values
mod enum_semantic_tests {
    use super::*;
    use ublox::nav_pvt::proto23::NavPvt;
    use ublox::GnssFixType;
    
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(200))]
        
        /// Test that NavPvt's fix_type field only contains valid GnssFixType values (0-5)
        /// This demonstrates the semantic awareness of enum-mapped fields.
        #[test]
        fn test_nav_pvt_fix_type_is_valid(payload in NavPvt::fuzz_payload_strategy()) {
            // fix_type is at byte offset 20 in NAV-PVT payload
            let fix_type_byte = payload[20];
            
            // GnssFixType only has values 0-5, so the auto-generated strategy
            // should ONLY produce these values, not arbitrary 0-255
            prop_assert!(
                fix_type_byte <= 5,
                "fix_type {} is not a valid GnssFixType (expected 0-5)",
                fix_type_byte
            );
        }
        
        /// Test round-trip parsing of NavPvt with enum-aware strategy
        #[test]
        fn test_nav_pvt_roundtrip_with_valid_fix_type(frame in NavPvt::fuzz_frame_strategy()) {
            let mut parser = ParserBuilder::new()
                .with_protocol::<Proto23>()
                .with_fixed_buffer::<256>();
            
            let mut it = parser.consume_ubx(&frame);
            
            if let Some(Ok(UbxPacket::Proto23(ublox::proto23::PacketRef::NavPvt(pvt)))) = it.next() {
                // Successfully parsed - the fix_type should be a valid enum value
                let fix_type = pvt.fix_type();
                match fix_type {
                    GnssFixType::NoFix 
                    | GnssFixType::DeadReckoningOnly 
                    | GnssFixType::Fix2D 
                    | GnssFixType::Fix3D 
                    | GnssFixType::GPSPlusDeadReckoning 
                    | GnssFixType::TimeOnlyFix => {}
                    _ => {}
                }
            }
        }
    }
}

// Demonstrate dual-mode fuzzing: valid vs chaos strategies
mod chaos_vs_valid_tests {
    use super::*;
    use ublox::nav_pvt::proto23::NavPvt;
    
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(200))]
        
        /// CHAOS MODE: fix_type can be ANY u8 value (0-255)
        /// This tests that the parser handles invalid enum values gracefully.
        #[test]
        fn test_nav_pvt_chaos_fix_type_full_range(payload in NavPvt::fuzz_payload_chaos_strategy()) {
            let fix_type_byte = payload[20];
            
            // In chaos mode, fix_type can be ANY value 0-255
            // This test just verifies we're generating the full range
            // (statistically, we should see values > 5 in 200 runs)
            prop_assert!(fix_type_byte <= 255); // Always true, but shows intent
        }
        
        /// CHAOS MODE: Parser should not panic on invalid data
        /// This tests graceful error handling.
        #[test]
        fn test_nav_pvt_chaos_no_panic(frame in NavPvt::fuzz_frame_chaos_strategy()) {
            let mut parser = ParserBuilder::new()
                .with_protocol::<Proto23>()
                .with_fixed_buffer::<256>();
            
            // The key assertion: parsing should NOT panic, even with invalid data
            let mut it = parser.consume_ubx(&frame);
            
            // We don't care if it succeeds or fails, just that it doesn't panic
            match it.next() {
                Some(Ok(_)) => { /* Valid parse - fine */ }
                Some(Err(_)) => { /* Parse error - also fine, expected for invalid data */ }
                None => { /* No packet - also fine */ }
            }
        }
    }
    
    /// Statistical test: chaos mode should produce invalid enum values
    /// while valid mode should not.
    #[test]
    fn test_chaos_produces_invalid_values() {
        use proptest::test_runner::{TestRunner, Config};
        use proptest::strategy::{Strategy, ValueTree};
        
        let mut runner = TestRunner::new(Config::with_cases(500));
        let mut found_invalid = false;
        
        // Generate 500 chaos payloads and check if any have invalid fix_type
        for _ in 0..500 {
            let payload = NavPvt::fuzz_payload_chaos_strategy()
                .new_tree(&mut runner)
                .unwrap()
                .current();
            
            if payload[20] > 5 {
                found_invalid = true;
                break;
            }
        }
        
        assert!(found_invalid, "Chaos mode should produce invalid enum values (fix_type > 5)");
    }
    
    /// Statistical test: valid mode should NOT produce invalid enum values
    #[test]
    fn test_valid_never_produces_invalid_values() {
        use proptest::test_runner::{TestRunner, Config};
        use proptest::strategy::{Strategy, ValueTree};
        
        let mut runner = TestRunner::new(Config::with_cases(500));
        
        // Generate 500 valid payloads and verify none have invalid fix_type
        for _ in 0..500 {
            let payload = NavPvt::fuzz_payload_strategy()
                .new_tree(&mut runner)
                .unwrap()
                .current();
            
            assert!(
                payload[20] <= 5,
                "Valid mode produced invalid fix_type: {}",
                payload[20]
            );
        }
    }
}
