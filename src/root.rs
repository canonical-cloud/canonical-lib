#![forbid(unsafe_code)]

#[path = "lib.rs"]
mod domain;
mod wire;

pub use canonical_interfaces as interfaces;
pub use domain::*;
pub use wire::{validate_wire_quote, WireValidationError};

/// Immutable `canonical-interfaces` revision used by this Cargo build.
pub const INTERFACES_REVISION: &str = "4c6ca63ca24fa214a1cb1a917ac27f1d5265916a";
