//! Traits for fuzz testing support.
//!
//! These traits enable auto-generated fuzz strategies to use
//! semantically valid values for enum-mapped fields.

/// Trait implemented by enums that can provide their valid raw values for fuzzing.
///
/// This is automatically implemented by `#[ubx_extend]` enums when the `fuzz` feature
/// is enabled, allowing fuzz strategies to generate only valid enum values rather
/// than any value in the underlying type's range.
pub trait UbxEnumFuzzable {
    /// The underlying raw type (typically u8)
    type Raw;
    
    /// Returns a slice of all valid raw values for this enum.
    ///
    /// For enums with `rest_reserved`, this returns only the explicitly
    /// defined variant values, not the reserved ones.
    fn valid_raw_values() -> &'static [Self::Raw];
}
