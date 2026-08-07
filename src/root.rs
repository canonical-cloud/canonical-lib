#![forbid(unsafe_code)]

#[path = "lib.rs"]
mod domain;
mod wire;

pub use canonical_interfaces as interfaces;
pub use domain::*;
pub use wire::{validate_wire_quote, WireValidationError};

/// Immutable `canonical-interfaces` revision used by this Cargo build.
pub const INTERFACES_REVISION: &str = "ec2c739092c955e4756d2d692ef225adf67647e4";
