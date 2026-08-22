#![forbid(unsafe_code)]

mod dispatch;
#[path = "lib.rs"]
mod domain;
mod issue;
mod wire;

pub use canonical_interfaces as interfaces;
pub use dispatch::{dispatch, OP_VALIDATE_QUOTE_REQUEST};
pub use domain::*;
pub use issue::{Issue, IssueKind, Outcome};
pub use wire::{validate_quote_request, validate_wire_quote, WireValidationError};

/// Immutable `canonical-interfaces` revision used by this Cargo build.
pub const INTERFACES_REVISION: &str = "4c6ca63ca24fa214a1cb1a917ac27f1d5265916a";
