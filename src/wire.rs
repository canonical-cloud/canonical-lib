//! Validation of the generated quote wire contract.
//!
//! Implements `conformance/README.md`: every failing field is reported, each
//! field yields at most one issue, and issues are ordered by schema property
//! declaration order. Issues name a field and a reason and never carry the
//! submitted value.

use std::collections::HashSet;

use crate::interfaces;
use crate::issue::{Issue, IssueKind, Outcome};

const FRAMEWORKS: &[&str] = &[
    "soc2_type_1",
    "soc2_type_2",
    "nist_csf_2",
    "nist_800_53",
    "hipaa",
    "iso_27001",
    "pci_dss_4",
    "fedramp",
    "gdpr",
    "custom",
];
const CURRENT_STAGES: &[&str] = &[
    "exploring",
    "readiness",
    "remediation",
    "audit_ready",
    "renewal",
];
const INFRASTRUCTURE: &[&str] = &[
    "aws",
    "azure",
    "gcp",
    "supabase",
    "on_prem",
    "colocation",
    "saas_only",
    "multi_cloud",
    "other",
];
const DATA_SENSITIVITY: &[&str] = &[
    "public",
    "internal",
    "confidential",
    "pii",
    "phi",
    "pci",
    "government_cui",
    "customer_secrets",
    "other",
];
const REVENUE_BANDS: &[&str] = &[
    "pre_revenue",
    "under_1m",
    "1m_10m",
    "10m_50m",
    "50m_250m",
    "over_250m",
    "prefer_not_to_say",
];

const MAX_SET_ITEMS: usize = 12;
const ANSWERS_VERSION: i64 = 1;

/// Validate a quote request, collecting every failing field.
///
/// Issues are pushed in schema property declaration order, so the caller never
/// has to sort: `organizationName`, `contactName`, `contactEmail`, `website`,
/// `employeeCount`, `annualRevenueBand`, `frameworks`, `currentStage`,
/// `infrastructure`, `dataSensitivity`, `targetDate`, `notes`, `contextKey`,
/// `answersVersion`. Boolean fields carry no rules.
#[must_use]
pub fn validate_quote_request(request: &interfaces::QuoteRequest) -> Outcome {
    let mut issues = Vec::new();

    push(
        &mut issues,
        "organizationName",
        text(&request.organization_name, 200),
    );
    push(&mut issues, "contactName", text(&request.contact_name, 160));
    push(&mut issues, "contactEmail", email(&request.contact_email));
    push(
        &mut issues,
        "website",
        request.website.as_deref().and_then(website),
    );
    push(
        &mut issues,
        "employeeCount",
        employee_count(request.employee_count),
    );
    push(
        &mut issues,
        "annualRevenueBand",
        request
            .annual_revenue_band
            .as_deref()
            .and_then(|band| one_of(band, REVENUE_BANDS)),
    );
    push(
        &mut issues,
        "frameworks",
        set(&request.frameworks, FRAMEWORKS),
    );
    push(
        &mut issues,
        "currentStage",
        one_of(&request.current_stage, CURRENT_STAGES),
    );
    push(
        &mut issues,
        "infrastructure",
        set(&request.infrastructure, INFRASTRUCTURE),
    );
    push(
        &mut issues,
        "dataSensitivity",
        set(&request.data_sensitivity, DATA_SENSITIVITY),
    );
    push(
        &mut issues,
        "targetDate",
        request
            .target_date
            .as_deref()
            .and_then(|date| reject_unless(is_iso_date(date))),
    );
    push(
        &mut issues,
        "notes",
        request
            .notes
            .as_deref()
            .and_then(|notes| too_long(notes, 5_000)),
    );
    push(
        &mut issues,
        "contextKey",
        request
            .context_key
            .as_deref()
            .and_then(|key| reject_unless(is_context_key(key))),
    );
    push(
        &mut issues,
        "answersVersion",
        (request.answers_version != ANSWERS_VERSION).then_some(IssueKind::UnsupportedVersion {
            expected: ANSWERS_VERSION,
        }),
    );

    Outcome(issues)
}

fn push(issues: &mut Vec<Issue>, field: &'static str, kind: Option<IssueKind>) {
    if let Some(kind) = kind {
        issues.push(Issue::new(field, kind));
    }
}

fn reject_unless(ok: bool) -> Option<IssueKind> {
    (!ok).then_some(IssueKind::Malformed)
}

fn char_count(value: &str) -> usize {
    // Unicode scalar values, not bytes or UTF-16 units, so Rust, Dart, and
    // TypeScript agree on any astral character.
    value.chars().count()
}

fn too_long(value: &str, limit: usize) -> Option<IssueKind> {
    let actual = char_count(value);
    (actual > limit).then_some(IssueKind::TooLong { limit, actual })
}

/// Required free text: missing, then untrimmed, then over-long.
fn text(value: &str, limit: usize) -> Option<IssueKind> {
    if value.trim().is_empty() {
        return Some(IssueKind::Missing);
    }
    if value.trim() != value {
        return Some(IssueKind::NotTrimmed);
    }
    too_long(value, limit)
}

fn email(value: &str) -> Option<IssueKind> {
    if value.trim().is_empty() {
        return Some(IssueKind::Missing);
    }
    if value.trim() != value {
        return Some(IssueKind::NotTrimmed);
    }
    if let Some(kind) = too_long(value, 320) {
        return Some(kind);
    }
    reject_unless(is_reasonable_email(value))
}

fn website(value: &str) -> Option<IssueKind> {
    if let Some(kind) = too_long(value, 2_048) {
        return Some(kind);
    }
    reject_unless(is_public_website(value))
}

fn employee_count(value: i64) -> Option<IssueKind> {
    const MIN: i64 = 1;
    const MAX: i64 = 1_000_000;
    (!(MIN..=MAX).contains(&value)).then_some(IssueKind::OutOfRange { min: MIN, max: MAX })
}

fn one_of(value: &str, allowed: &[&str]) -> Option<IssueKind> {
    (!allowed.contains(&value)).then_some(IssueKind::NotAllowed)
}

/// Set-valued field: too few, then too many, then an unknown value, then a
/// duplicate. `not_allowed` precedes `duplicate` because the scan stops at the
/// first offending item.
fn set(values: &[String], allowed: &[&str]) -> Option<IssueKind> {
    let actual = values.len();
    if actual < 1 {
        return Some(IssueKind::TooFew { limit: 1, actual });
    }
    if actual > MAX_SET_ITEMS {
        return Some(IssueKind::TooMany {
            limit: MAX_SET_ITEMS,
            actual,
        });
    }
    let mut observed = HashSet::with_capacity(actual);
    for value in values {
        if !allowed.contains(&value.as_str()) {
            return Some(IssueKind::NotAllowed);
        }
        if !observed.insert(value.as_str()) {
            return Some(IssueKind::Duplicate);
        }
    }
    None
}

fn is_reasonable_email(value: &str) -> bool {
    let Some((local, domain)) = value.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && !domain.is_empty()
        && !domain.contains('@')
        && !local.chars().any(char::is_whitespace)
        && !domain.chars().any(char::is_whitespace)
        && domain.contains('.')
}

fn is_public_website(value: &str) -> bool {
    let remainder = value
        .strip_prefix("https://")
        .or_else(|| value.strip_prefix("http://"));
    remainder.is_some_and(|host_and_path| {
        let host = host_and_path.split('/').next().unwrap_or_default();
        !host.is_empty()
            && host.contains('.')
            && !host.chars().any(char::is_whitespace)
            && !host.contains('@')
    })
}

fn is_iso_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return false;
    }
    if !bytes
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != 4 && *index != 7)
        .all(|(_, byte)| byte.is_ascii_digit())
    {
        return false;
    }

    let year = value[0..4].parse::<u16>().ok();
    let month = value[5..7].parse::<u8>().ok();
    let day = value[8..10].parse::<u8>().ok();
    matches!(year, Some(1..=9_999)) && matches!(month, Some(1..=12)) && matches!(day, Some(1..=31))
}

fn is_context_key(value: &str) -> bool {
    // Constrained to ASCII, so the 128 limit is unambiguous in every language.
    if value.is_empty() || value.len() > 128 || !value.is_ascii() {
        return false;
    }
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_lowercase() || first.is_ascii_digit())
        && chars.all(|character| {
            character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || matches!(character, '.' | '_' | '-')
        })
}

#[cfg(test)]
mod tests {
    use crate::INTERFACES_REVISION;

    /// The pinned revision is written in two places — `Cargo.toml` and
    /// `INTERFACES_REVISION` — and a bump that updates one but not the other
    /// would leave the constant lying about what this build resolved. The
    /// conformance corpus cannot see this, so it stays a unit test.
    #[test]
    fn interfaces_revision_matches_the_cargo_pin() {
        let manifest = include_str!("../Cargo.toml");
        let pinned = manifest
            .lines()
            .find(|line| line.contains("canonical-interfaces") && line.contains("rev = "))
            .and_then(|line| line.split("rev = \"").nth(1))
            .and_then(|rest| rest.split('"').next())
            .expect("Cargo.toml has no pinned canonical-interfaces revision");

        assert_eq!(
            pinned, INTERFACES_REVISION,
            "INTERFACES_REVISION has drifted from the Cargo.toml pin",
        );
        assert_eq!(INTERFACES_REVISION.len(), 40, "not a full commit SHA");
        assert!(
            INTERFACES_REVISION.chars().all(|c| c.is_ascii_hexdigit()),
            "not a hexadecimal commit SHA",
        );
    }
}
