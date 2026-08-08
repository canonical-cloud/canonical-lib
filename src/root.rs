#![forbid(unsafe_code)]

#[path = "lib.rs"]
mod domain;
mod dispatch;
mod issue;
mod wire;

pub use canonical_interfaces as interfaces;
pub use domain::*;
pub use dispatch::{dispatch, OP_VALIDATE_QUOTE_REQUEST};
pub use issue::{Issue, IssueKind, Outcome};
pub use wire::validate_quote_request;

/// Immutable `canonical-interfaces` revision used by this Cargo build.
pub const INTERFACES_REVISION: &str = "4c6ca63ca24fa214a1cb1a917ac27f1d5265916a";
