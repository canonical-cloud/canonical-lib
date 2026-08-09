//! The shared error vocabulary from `conformance/README.md`.
//!
//! Serialization is written by hand rather than derived because the corpus
//! compares **exact bytes**: keys must be `field`, `kind`, then that kind's
//! parameters in the documented order. A derived impl would not express the
//! per-kind parameter ordering, and `serde_json::Value` would sort the keys
//! alphabetically (`actual`, `field`, `kind`, `limit`), which is wrong.

use serde::ser::{Serialize, SerializeMap, Serializer};

/// Why a field was rejected. Parameters are lengths and bounds only — never the
/// submitted value, which may carry customer identity or infrastructure detail.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IssueKind {
    /// Required value absent, or empty once trimmed.
    Missing,
    /// Leading or trailing whitespace.
    NotTrimmed,
    /// Exceeds the maximum character count.
    TooLong { limit: usize, actual: usize },
    /// Array below `minItems`.
    TooFew { limit: usize, actual: usize },
    /// Array above `maxItems`.
    TooMany { limit: usize, actual: usize },
    /// Repeated item in a set-valued field.
    Duplicate,
    /// Value outside the permitted set.
    NotAllowed,
    /// Fails a format rule (email, URL, date, key).
    Malformed,
    /// Numeric value outside its bounds.
    OutOfRange { min: i64, max: i64 },
    /// `answersVersion` is not the supported version.
    UnsupportedVersion { expected: i64 },
}

impl IssueKind {
    /// The wire `kind` string. Part of the cross-language contract.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Missing => "missing",
            Self::NotTrimmed => "not_trimmed",
            Self::TooLong { .. } => "too_long",
            Self::TooFew { .. } => "too_few",
            Self::TooMany { .. } => "too_many",
            Self::Duplicate => "duplicate",
            Self::NotAllowed => "not_allowed",
            Self::Malformed => "malformed",
            Self::OutOfRange { .. } => "out_of_range",
            Self::UnsupportedVersion { .. } => "unsupported_version",
        }
    }
}

/// One rejected field. A field yields at most one issue — the first rule it
/// fails — so that "how many issues does a bad field produce" is not a fourth
/// thing the three implementations must agree on.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Issue {
    pub field: &'static str,
    pub kind: IssueKind,
}

impl Issue {
    #[must_use]
    pub const fn new(field: &'static str, kind: IssueKind) -> Self {
        Self { field, kind }
    }
}

impl Serialize for Issue {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("field", self.field)?;
        map.serialize_entry("kind", self.kind.as_str())?;
        match self.kind {
            IssueKind::TooLong { limit, actual }
            | IssueKind::TooFew { limit, actual }
            | IssueKind::TooMany { limit, actual } => {
                map.serialize_entry("limit", &limit)?;
                map.serialize_entry("actual", &actual)?;
            }
            IssueKind::OutOfRange { min, max } => {
                map.serialize_entry("min", &min)?;
                map.serialize_entry("max", &max)?;
            }
            IssueKind::UnsupportedVersion { expected } => {
                map.serialize_entry("expected", &expected)?;
            }
            IssueKind::Missing
            | IssueKind::NotTrimmed
            | IssueKind::Duplicate
            | IssueKind::NotAllowed
            | IssueKind::Malformed => {}
        }
        map.end()
    }
}

/// The response envelope. `status` precedes `errors`, and `errors` is omitted
/// entirely when empty, matching `{"status":"ok"}` in the corpus.
#[derive(Debug)]
pub struct Outcome(pub Vec<Issue>);

impl Outcome {
    #[must_use]
    pub fn is_accepted(&self) -> bool {
        self.0.is_empty()
    }
}

impl Serialize for Outcome {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(None)?;
        if self.0.is_empty() {
            map.serialize_entry("status", "ok")?;
        } else {
            map.serialize_entry("status", "rejected")?;
            map.serialize_entry("errors", &self.0)?;
        }
        map.end()
    }
}
