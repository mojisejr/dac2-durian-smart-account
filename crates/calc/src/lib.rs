#![forbid(unsafe_code)]

//! Pure calculation boundary for DAC2.
//!
//! Product calculations begin in slice 2. This crate intentionally contains no
//! I/O, async runtime, or feature gate.

use rust_decimal::Decimal;

/// Compile-time witness that the shared decimal type is available to both
/// native and WebAssembly builds.
pub const ZERO: Decimal = Decimal::ZERO;
