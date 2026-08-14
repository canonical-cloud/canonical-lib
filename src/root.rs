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
pub use wire::validate_quote_request;

/// Immutable `canonical-interfaces` revision used by this Cargo build.
pub const INTERFACES_REVISION: &str = "0cab33c2b2a494d2368ef1da0ebe5d614b3a96ef";
