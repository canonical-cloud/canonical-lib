use std::collections::HashSet;
use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::interfaces;

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

/// Validate the current generated quote contract without copying transport
/// structs into this crate. Errors identify only the field, never its value.
pub fn validate_wire_quote(request: &interfaces::QuoteRequest) -> Result<(), WireValidationError> {
    validate_text("organizationName", &request.organization_name, 1, 200)?;
    validate_text("contactName", &request.contact_name, 1, 160)?;
    if !is_reasonable_email(&request.contact_email) {
        return Err(WireValidationError::InvalidField("contactEmail"));
    }
    if request
        .website
        .as_deref()
        .is_some_and(|website| char_count(website) > 2_048 || !is_public_website(website))
    {
        return Err(WireValidationError::InvalidField("website"));
    }
    if !(1..=1_000_000).contains(&request.employee_count) {
        return Err(WireValidationError::InvalidField("employeeCount"));
    }
    if request
        .annual_revenue_band
        .as_deref()
        .is_some_and(|band| !REVENUE_BANDS.contains(&band))
    {
        return Err(WireValidationError::InvalidField("annualRevenueBand"));
    }

    validate_unique_values("frameworks", &request.frameworks, 1, 12, FRAMEWORKS)?;
    if !CURRENT_STAGES.contains(&request.current_stage.as_str()) {
        return Err(WireValidationError::InvalidField("currentStage"));
    }
    validate_unique_values(
        "infrastructure",
        &request.infrastructure,
        1,
        12,
        INFRASTRUCTURE,
    )?;
    validate_unique_values(
        "dataSensitivity",
        &request.data_sensitivity,
        1,
        12,
        DATA_SENSITIVITY,
    )?;

    if request
        .target_date
        .as_deref()
        .is_some_and(|date| !is_iso_date(date))
    {
        return Err(WireValidationError::InvalidField("targetDate"));
    }
    if request
        .notes
        .as_deref()
        .is_some_and(|notes| char_count(notes) > 5_000)
    {
        return Err(WireValidationError::InvalidField("notes"));
    }
    if request
        .context_key
        .as_deref()
        .is_some_and(|key| !is_context_key(key))
    {
        return Err(WireValidationError::InvalidField("contextKey"));
    }
    if request.answers_version != 1 {
        return Err(WireValidationError::InvalidField("answersVersion"));
    }

    Ok(())
}

fn validate_text(
    field: &'static str,
    value: &str,
    minimum_chars: usize,
    maximum_chars: usize,
) -> Result<(), WireValidationError> {
    let length = char_count(value);
    if value.trim() != value || !(minimum_chars..=maximum_chars).contains(&length) {
        return Err(WireValidationError::InvalidField(field));
    }
    Ok(())
}

fn validate_unique_values(
    field: &'static str,
    values: &[String],
    minimum_items: usize,
    maximum_items: usize,
    allowed: &[&str],
) -> Result<(), WireValidationError> {
    if !(minimum_items..=maximum_items).contains(&values.len()) {
        return Err(WireValidationError::InvalidField(field));
    }

    let mut observed = HashSet::with_capacity(values.len());
    for value in values {
        if !allowed.contains(&value.as_str()) {
            return Err(WireValidationError::InvalidField(field));
        }
        if !observed.insert(value.as_str()) {
            return Err(WireValidationError::DuplicateItem(field));
        }
    }
    Ok(())
}

fn char_count(value: &str) -> usize {
    value.chars().count()
}

fn is_reasonable_email(value: &str) -> bool {
    if value.trim() != value || value.is_empty() || char_count(value) > 320 {
        return false;
    }
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WireValidationError {
    InvalidField(&'static str),
    DuplicateItem(&'static str),
}

impl Display for WireValidationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidField(field) => write!(formatter, "invalid wire field: {field}"),
            Self::DuplicateItem(field) => {
                write!(formatter, "duplicate item in wire field: {field}")
            }
        }
    }
}

impl Error for WireValidationError {}

#[cfg(test)]
mod tests {
    use super::{validate_wire_quote, WireValidationError};
    use crate::{interfaces, INTERFACES_REVISION};

    fn valid_wire_request() -> interfaces::QuoteRequest {
        interfaces::QuoteRequest {
            organization_name: "Example Incorporated".into(),
            contact_name: "Jordan Example".into(),
            contact_email: "jordan@example.com".into(),
            website: Some("https://example.com".into()),
            employee_count: 42,
            annual_revenue_band: Some("1m_10m".into()),
            frameworks: vec!["soc2_type_2".into(), "iso_27001".into()],
            current_stage: "readiness".into(),
            infrastructure: vec!["aws".into(), "supabase".into()],
            data_sensitivity: vec!["confidential".into(), "pii".into()],
            target_date: Some("2027-01-15".into()),
            has_security_program: true,
            has_policies: true,
            has_risk_assessment: false,
            has_incident_response_plan: true,
            has_vendor_management: false,
            notes: Some("Initial readiness estimate".into()),
            context_key: Some("quote.default-v1".into()),
            answers_version: 1,
        }
    }

    #[test]
    fn validates_the_generated_wire_contract() {
        assert_eq!(validate_wire_quote(&valid_wire_request()), Ok(()));
        assert_eq!(INTERFACES_REVISION.len(), 40);
    }

    #[test]
    fn rejects_unknown_and_duplicate_values_without_echoing_input() {
        let mut request = valid_wire_request();
        request.frameworks.push("unreviewed_framework".into());
        assert_eq!(
            validate_wire_quote(&request),
            Err(WireValidationError::InvalidField("frameworks"))
        );

        let mut request = valid_wire_request();
        request.infrastructure.push("aws".into());
        assert_eq!(
            validate_wire_quote(&request),
            Err(WireValidationError::DuplicateItem("infrastructure"))
        );
    }

    #[test]
    fn rejects_malformed_metadata() {
        let mut request = valid_wire_request();
        request.context_key = Some("UPPERCASE".into());
        request.answers_version = 2;
        assert_eq!(
            validate_wire_quote(&request),
            Err(WireValidationError::InvalidField("contextKey"))
        );

        request.context_key = Some("valid-key".into());
        assert_eq!(
            validate_wire_quote(&request),
            Err(WireValidationError::InvalidField("answersVersion"))
        );
    }
}
