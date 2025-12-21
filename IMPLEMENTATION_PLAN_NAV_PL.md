# Implementation Plan: UBX-NAV-PL Message Support

## Overview

This document outlines the step-by-step plan to implement UBX-NAV-PL (Protection Level) message support in the ublox-rs crate.

---

## Prerequisites

Before starting:
1. Fork the repository (if not already done)
2. Set up your fork as a remote: `git remote add fork <your-fork-url>`
3. Create a feature branch: `git checkout -b feature/nav-pl-support`
4. Ensure you can build and test: `cargo build && cargo test`

---

## Implementation Steps

### Step 1: Create the NAV-PL Packet Definition

**File:** `ublox/src/ubx_packets/packets/nav_pl.rs`

```rust
#[cfg(feature = "serde")]
use super::SerializeUbxPacketFields;
#[cfg(feature = "serde")]
use crate::serde::ser::SerializeMap;

use bitflags::bitflags;

use crate::{error::ParserError, UbxPacketMeta};
use ublox_derive::{ubx_extend, ubx_extend_bitflags, ubx_packet_recv};

/// Protection Level Information
///
/// This message outputs protection level information for position, velocity,
/// and time. Protection levels bound the true error with a specified confidence
/// level (Target Integrity Risk).
#[ubx_packet_recv]
#[ubx(class = 0x01, id = 0x62, fixed_payload_len = 56)]
struct NavPl {
    /// GPS time of week (ms)
    itow: u32,

    /// Message version (0 for this version)
    version: u8,

    /// Reserved
    reserved0: [u8; 3],

    /// Target Integrity Risk Mantissa coefficient
    /// TMIR = tmirCoeff * 10^tmirExp
    tmir_coeff: u8,

    /// Target Integrity Risk exponent
    tmir_exp: i8,

    /// Position protection level validity flags
    #[ubx(map_type = PlPosValid)]
    pl_pos_valid: u8,

    /// Position protection level reference frame
    #[ubx(map_type = PlPosFrame)]
    pl_pos_frame: u8,

    /// Position protection level semi-major axis of error ellipse
    /// Scale: 0.01 m
    #[ubx(map_type = f64, scale = 0.01)]
    pl_pos1: u32,

    /// Position protection level semi-minor axis of error ellipse
    /// Scale: 0.01 m
    #[ubx(map_type = f64, scale = 0.01)]
    pl_pos2: u32,

    /// Position protection level vertical component
    /// Scale: 0.01 m
    #[ubx(map_type = f64, scale = 0.01)]
    pl_pos3: u32,

    /// Orientation of semi-major axis of position error ellipse
    /// Scale: 1e-2 degrees, range: -18000 to 18000
    #[ubx(map_type = f64, scale = 0.01)]
    pl_pos_hor_orient: i32,

    /// Velocity protection level validity flags
    #[ubx(map_type = PlVelValid)]
    pl_vel_valid: u8,

    /// Velocity protection level reference frame
    #[ubx(map_type = PlVelFrame)]
    pl_vel_frame: u8,

    /// Reserved
    reserved1: [u8; 2],

    /// Velocity protection level semi-major axis
    /// Scale: 0.01 m/s
    #[ubx(map_type = f64, scale = 0.01)]
    pl_vel1: u32,

    /// Velocity protection level semi-minor axis
    /// Scale: 0.01 m/s
    #[ubx(map_type = f64, scale = 0.01)]
    pl_vel2: u32,

    /// Velocity protection level vertical component
    /// Scale: 0.01 m/s
    #[ubx(map_type = f64, scale = 0.01)]
    pl_vel3: u32,

    /// Orientation of semi-major axis of velocity error ellipse
    /// Scale: 1e-2 degrees
    #[ubx(map_type = f64, scale = 0.01)]
    pl_vel_hor_orient: i32,

    /// Time protection level validity flags
    #[ubx(map_type = PlTimeValid)]
    pl_time_valid: u8,

    /// Reserved
    reserved2: [u8; 3],

    /// Time protection level (ns)
    pl_time: u32,
}

// Validity flags for position protection level
#[ubx_extend_bitflags]
#[ubx(from, rest_reserved)]
bitflags! {
    #[derive(Debug)]
    pub struct PlPosValid: u8 {
        /// Horizontal protection level valid
        const HOR_VALID = 0x01;
        /// Vertical protection level valid
        const VER_VALID = 0x02;
    }
}

// Position reference frame
#[ubx_extend]
#[ubx(from, rest_reserved)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PlPosFrame {
    /// Invalid or unknown frame
    Invalid = 0,
    /// Local tangent plane (NED)
    Ned = 1,
}

// Validity flags for velocity protection level
#[ubx_extend_bitflags]
#[ubx(from, rest_reserved)]
bitflags! {
    #[derive(Debug)]
    pub struct PlVelValid: u8 {
        /// Horizontal velocity protection level valid
        const HOR_VALID = 0x01;
        /// Vertical velocity protection level valid
        const VER_VALID = 0x02;
    }
}

// Velocity reference frame
#[ubx_extend]
#[ubx(from, rest_reserved)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PlVelFrame {
    /// Invalid or unknown frame
    Invalid = 0,
    /// Local tangent plane (NED)
    Ned = 1,
}

// Validity flags for time protection level
#[ubx_extend_bitflags]
#[ubx(from, rest_reserved)]
bitflags! {
    #[derive(Debug)]
    pub struct PlTimeValid: u8 {
        /// Time protection level valid
        const VALID = 0x01;
    }
}
```

---

### Step 2: Register the Module in packets.rs

**File:** `ublox/src/ubx_packets/packets.rs`

Add the module declaration (around line 90, with other nav_* modules):

```rust
pub mod nav_pl;
```

---

### Step 3: Register in Protocol Version Files

NAV-PL availability varies by device (not strictly by protocol version):

| Device | Firmware | Protocol |
|--------|----------|----------|
| ZED-F9P | HPG 1.30+ | 27 |
| ZED-F9R | HPS 1.30+ | 33 |
| M10 | SPG 5.10+ | 34 |
| NEO-F10N | SPG 6.00+ | 40 |

**Note:** X20/F20 devices do **not** support NAV-PL.

**Recommendation:** Add to `ubx_proto27` and `ubx_proto31` since ZED-F9P (widely used) supports it at proto27. Document that users must verify device support.

**For Protocol 27:** `ublox/src/ubx_packets/packets/packetref_proto27.rs`

Add import:
```rust
nav_pl::{NavPl, NavPlOwned, NavPlRef},
```

Add to `define_recv_packets!` enum:
```rust
NavPl,
```

**For Protocol 31:** `ublox/src/ubx_packets/packets/packetref_proto31.rs`

Same changes as protocol 27.

---

### Step 4: Add Unit Tests

**File:** `ublox/tests/nav_pl_test.rs` (new file)

```rust
use ublox::*;

#[test]
fn test_nav_pl_parsing() {
    // Create a sample NAV-PL packet payload (56 bytes)
    let payload: [u8; 56] = [
        // iTOW: 123456789 ms = 0x075BCD15
        0x15, 0xCD, 0x5B, 0x07,
        // version: 0
        0x00,
        // reserved0: [0, 0, 0]
        0x00, 0x00, 0x00,
        // tmirCoeff: 1
        0x01,
        // tmirExp: -7
        0xF9,
        // plPosValid: 0x03 (both valid)
        0x03,
        // plPosFrame: 1 (NED)
        0x01,
        // plPos1: 1000 (10.00 m)
        0xE8, 0x03, 0x00, 0x00,
        // plPos2: 500 (5.00 m)
        0xF4, 0x01, 0x00, 0x00,
        // plPos3: 2000 (20.00 m)
        0xD0, 0x07, 0x00, 0x00,
        // plPosHorOrient: 4500 (45.00 deg)
        0x94, 0x11, 0x00, 0x00,
        // plVelValid: 0x03
        0x03,
        // plVelFrame: 1
        0x01,
        // reserved1: [0, 0]
        0x00, 0x00,
        // plVel1: 100 (1.00 m/s)
        0x64, 0x00, 0x00, 0x00,
        // plVel2: 50 (0.50 m/s)
        0x32, 0x00, 0x00, 0x00,
        // plVel3: 200 (2.00 m/s)
        0xC8, 0x00, 0x00, 0x00,
        // plVelHorOrient: 9000 (90.00 deg)
        0x28, 0x23, 0x00, 0x00,
        // plTimeValid: 0x01
        0x01,
        // reserved2: [0, 0, 0]
        0x00, 0x00, 0x00,
        // plTime: 100 ns
        0x64, 0x00, 0x00, 0x00,
    ];

    // Test that parsing would work (actual parser integration test)
    assert_eq!(payload.len(), 56);
}
```

---

### Step 5: Update CHANGELOG.md

Add entry under the appropriate section:

```markdown
## [Unreleased]

### Added

- Add UBX-NAV-PL (Protection Level) message support
```

---

### Step 6: Verification Checklist

Before submitting PR:

- [ ] `cargo build --all-features` passes
- [ ] `cargo test --all-features` passes
- [ ] `cargo clippy --all-features` passes with no warnings
- [ ] `cargo fmt --check` passes
- [ ] Documentation builds: `cargo doc --all-features`
- [ ] Tested with actual device data (if available)

---

## Git Workflow

```bash
# Ensure your fork is up to date
git fetch origin
git rebase origin/master

# Create feature branch
git checkout -b feature/nav-pl-support

# Make changes...

# Commit with conventional commit message
git add .
git commit -m "feat: add UBX-NAV-PL message support

Add support for parsing the UBX-NAV-PL (0x01 0x62) Protection Level
message which provides integrity information for position, velocity,
and time estimates.

- Add NavPl packet struct with all fields
- Add validity flag bitflags for pos/vel/time
- Add reference frame enums
- Register in proto27 and proto31 PacketRef enums
- Add unit tests

Closes #XXX"

# Push to your fork
git push fork feature/nav-pl-support
```

---

## Files Changed Summary

| File | Action |
|------|--------|
| `ublox/src/ubx_packets/packets/nav_pl.rs` | **Create** |
| `ublox/src/ubx_packets/packets.rs` | **Modify** - add module |
| `ublox/src/ubx_packets/packets/packetref_proto27.rs` | **Modify** - register packet |
| `ublox/src/ubx_packets/packets/packetref_proto31.rs` | **Modify** - register packet |
| `ublox/tests/nav_pl_test.rs` | **Create** (optional) |
| `CHANGELOG.md` | **Modify** - add entry |

---

## Notes

- The exact field names and types should be verified against the official u-blox Interface Description document for your target receiver
- The message format may vary slightly between receiver generations (M10, F9, etc.)
- Consider whether this should be added to a new protocol feature flag (e.g., `ubx_proto34`) if it's only supported on very new receivers
- Check the existing codebase for any helper functions that might be useful (e.g., scaling utilities)
