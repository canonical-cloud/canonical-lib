//! The single entry point the conformance corpus drives.
//!
//! One dispatch table with one error vocabulary, rather than a typed signature
//! per rule, so there is exactly one thing to conformance-test per language.
//! `op` strings and error `kind` strings are a compatibility surface: renaming
//! one breaks every consumer.

use serde::Deserialize;

use crate::interfaces;
use crate::issue::{Issue, IssueKind, Outcome};

/// Ops understood by [`dispatch`]. Unknown ops are reported rather than
/// panicking, so a newer caller against an older build fails legibly.
pub const OP_VALIDATE_QUOTE_REQUEST: &str = "validate_quote_request";

#[derive(Deserialize)]
struct Envelope {
    op: String,
    #[serde(default)]
    request: Option<serde_json::Value>,
}

/// Take a request envelope as JSON, return the response as JSON.
///
/// The corpus compares exact bytes, so this returns a `String` rather than a
/// structured value: any re-serialization by the caller could reorder keys and
/// silently break the contract it is meant to prove.
#[must_use]
pub fn dispatch(request_json: &str) -> String {
    let outcome = match dispatch_outcome(request_json) {
        Ok(outcome) => outcome,
        Err(kind) => Outcome(vec![Issue::new("op", kind)]),
    };
    serde_json::to_string(&outcome).unwrap_or_else(|_| {
        // Outcome serializes only &'static str and integers, so this is
        // unreachable; degrade to a valid rejection rather than panicking in a
        // library that may be running inside someone's request handler.
        String::from(r#"{"status":"rejected","errors":[{"field":"op","kind":"malformed"}]}"#)
    })
}

fn dispatch_outcome(request_json: &str) -> Result<Outcome, IssueKind> {
    let envelope: Envelope =
        serde_json::from_str(request_json).map_err(|_| IssueKind::Malformed)?;
    if envelope.op != OP_VALIDATE_QUOTE_REQUEST {
        return Err(IssueKind::NotAllowed);
    }
    let payload = envelope.request.ok_or(IssueKind::Missing)?;
    let request: interfaces::QuoteRequest =
        serde_json::from_value(payload).map_err(|_| IssueKind::Malformed)?;
    Ok(crate::validate_quote_request(&request))
}
