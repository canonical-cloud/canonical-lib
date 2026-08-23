#![forbid(unsafe_code)]

use std::collections::HashSet;
use std::error::Error;
use std::fmt::{Display, Formatter};

/// The Zed package that owns generated wire-format contracts.
pub const INTERFACES_PACKAGE: &str = "canonical-cloud/canonical-interfaces";

/// Readiness programs a client can prepare for. canonical.plus prepares
/// clients for independent review against these frameworks; formal
/// determinations are made by independent auditors, assessors, and
/// regulators, never by canonical.plus itself.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Framework {
    Soc2,
    NistCsf,
    Nist80053,
    Hipaa,
    Iso27001,
    PciDss,
    CisControls,
    Cmmc,
    CsaCcm,
    Dora,
    FedRamp,
    Iso22301,
    Iso27701,
    Nis2,
    Custom,
}

impl Framework {
    /// Every framework variant, for iteration and round-trip checks.
    pub const ALL: [Self; 15] = [
        Self::Soc2,
        Self::NistCsf,
        Self::Nist80053,
        Self::Hipaa,
        Self::Iso27001,
        Self::PciDss,
        Self::CisControls,
        Self::Cmmc,
        Self::CsaCcm,
        Self::Dora,
        Self::FedRamp,
        Self::Iso22301,
        Self::Iso27701,
        Self::Nis2,
        Self::Custom,
    ];

    #[must_use]
    pub const fn as_slug(self) -> &'static str {
        match self {
            Self::Soc2 => "soc2",
            Self::NistCsf => "nist-csf",
            Self::Nist80053 => "nist-800-53",
            Self::Hipaa => "hipaa",
            Self::Iso27001 => "iso-27001",
            Self::PciDss => "pci-dss",
            Self::CisControls => "cis-controls",
            Self::Cmmc => "cmmc",
            Self::CsaCcm => "csa-ccm",
            Self::Dora => "dora",
            Self::FedRamp => "fedramp",
            Self::Iso22301 => "iso-22301",
            Self::Iso27701 => "iso-27701",
            Self::Nis2 => "nis2",
            Self::Custom => "custom",
        }
    }

    /// Inverse of [`Framework::as_slug`]. Accepts exactly the slugs that
    /// `as_slug` produces; anything else returns `None`.
    #[must_use]
    pub fn from_slug(slug: &str) -> Option<Self> {
        match slug {
            "soc2" => Some(Self::Soc2),
            "nist-csf" => Some(Self::NistCsf),
            "nist-800-53" => Some(Self::Nist80053),
            "hipaa" => Some(Self::Hipaa),
            "iso-27001" => Some(Self::Iso27001),
            "pci-dss" => Some(Self::PciDss),
            "cis-controls" => Some(Self::CisControls),
            "cmmc" => Some(Self::Cmmc),
            "csa-ccm" => Some(Self::CsaCcm),
            "dora" => Some(Self::Dora),
            "fedramp" => Some(Self::FedRamp),
            "iso-22301" => Some(Self::Iso22301),
            "iso-27701" => Some(Self::Iso27701),
            "nis2" => Some(Self::Nis2),
            "custom" => Some(Self::Custom),
            _ => None,
        }
    }
}

/// Customer characteristics used to size a readiness engagement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrganizationProfile {
    pub legal_name: String,
    pub employee_count: u32,
    pub industry: String,
    pub countries: Vec<String>,
}

/// Domain-level quote intake. Transport-specific JSON types live in
/// `canonical-interfaces` and should be adapted into this structure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuoteRequest {
    pub organization: OrganizationProfile,
    pub frameworks: Vec<Framework>,
    pub target_date: Option<String>,
    pub notes: Option<String>,
}

impl QuoteRequest {
    /// Validate bounded, domain-level invariants before persistence or model use.
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.organization.legal_name.trim().is_empty() {
            return Err(ValidationError::MissingLegalName);
        }
        if self.organization.industry.trim().is_empty() {
            return Err(ValidationError::MissingIndustry);
        }
        if self.organization.employee_count == 0 {
            return Err(ValidationError::InvalidEmployeeCount);
        }
        if self.organization.countries.len() > 64 {
            return Err(ValidationError::TooManyCountries);
        }
        if self.frameworks.is_empty() {
            return Err(ValidationError::MissingFramework);
        }

        let mut observed = HashSet::with_capacity(self.frameworks.len());
        for framework in &self.frameworks {
            if !observed.insert(*framework) {
                return Err(ValidationError::DuplicateFramework(*framework));
            }
        }

        if self
            .notes
            .as_deref()
            .is_some_and(|notes| notes.len() > 32_768)
        {
            return Err(ValidationError::NotesTooLarge);
        }

        Ok(())
    }
}

/// Context assembled from a reviewed Markdown file and one owner-scoped
/// `canonical_context` database record serialized as JSON.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalysisContext {
    pub markdown: String,
    pub database_record_json: String,
}

impl AnalysisContext {
    /// Render a deterministic model input while enforcing a caller-selected
    /// maximum. Callers should choose a limit below the model and logging caps.
    pub fn render(&self, maximum_bytes: usize) -> Result<String, ValidationError> {
        if self.markdown.trim().is_empty() {
            return Err(ValidationError::MissingMarkdownContext);
        }
        if self.database_record_json.trim().is_empty() {
            return Err(ValidationError::MissingDatabaseContext);
        }

        let rendered = format!(
            "# Reviewed Markdown Context\n\n{}\n\n# Owner-Scoped Database Context\n\n```json\n{}\n```\n",
            self.markdown.trim(),
            self.database_record_json.trim()
        );
        if rendered.len() > maximum_bytes {
            return Err(ValidationError::ContextTooLarge {
                actual: rendered.len(),
                maximum: maximum_bytes,
            });
        }
        Ok(rendered)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValidationError {
    MissingLegalName,
    MissingIndustry,
    InvalidEmployeeCount,
    TooManyCountries,
    MissingFramework,
    DuplicateFramework(Framework),
    NotesTooLarge,
    MissingMarkdownContext,
    MissingDatabaseContext,
    ContextTooLarge { actual: usize, maximum: usize },
}

impl Display for ValidationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingLegalName => formatter.write_str("organization legal name is required"),
            Self::MissingIndustry => formatter.write_str("organization industry is required"),
            Self::InvalidEmployeeCount => {
                formatter.write_str("employee count must be greater than zero")
            }
            Self::TooManyCountries => formatter.write_str("at most 64 countries are allowed"),
            Self::MissingFramework => {
                formatter.write_str("at least one compliance framework is required")
            }
            Self::DuplicateFramework(framework) => {
                write!(formatter, "duplicate framework: {}", framework.as_slug())
            }
            Self::NotesTooLarge => formatter.write_str("notes exceed 32768 bytes"),
            Self::MissingMarkdownContext => formatter.write_str("Markdown context is required"),
            Self::MissingDatabaseContext => {
                formatter.write_str("database context record is required")
            }
            Self::ContextTooLarge { actual, maximum } => {
                write!(
                    formatter,
                    "analysis context is {actual} bytes; maximum is {maximum}"
                )
            }
        }
    }
}

impl Error for ValidationError {}

#[cfg(test)]
mod tests {
    use super::{AnalysisContext, Framework, OrganizationProfile, QuoteRequest, ValidationError};

    fn valid_request() -> QuoteRequest {
        QuoteRequest {
            organization: OrganizationProfile {
                legal_name: "Example Incorporated".into(),
                employee_count: 42,
                industry: "Software".into(),
                countries: vec!["US".into(), "PE".into()],
            },
            frameworks: vec![Framework::Soc2, Framework::Hipaa],
            target_date: Some("2027-01-15".into()),
            notes: None,
        }
    }

    #[test]
    fn accepts_a_bounded_quote_request() {
        assert_eq!(valid_request().validate(), Ok(()));
    }

    #[test]
    fn framework_slugs_round_trip_for_all_variants() {
        for framework in Framework::ALL {
            let slug = framework.as_slug();
            assert_eq!(
                Framework::from_slug(slug),
                Some(framework),
                "slug {slug} should round-trip"
            );
        }
    }

    #[test]
    fn framework_slugs_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for framework in Framework::ALL {
            assert!(seen.insert(framework.as_slug()), "duplicate slug");
        }
        assert_eq!(seen.len(), Framework::ALL.len());
    }

    #[test]
    fn from_slug_rejects_unknown_and_near_miss_slugs() {
        for slug in [
            "",
            "SOC2",
            "soc-2",
            "nist_csf",
            "iso27001",
            "cis-controls-8.1",
            "cmmc-2.0",
            "dora-2022-2554",
            "fedramp-rev5",
            "nis2-2022-2555",
            "not-a-framework",
        ] {
            assert_eq!(Framework::from_slug(slug), None, "slug {slug:?}");
        }
    }

    #[test]
    fn rejects_duplicate_frameworks() {
        let mut request = valid_request();
        request.frameworks = vec![Framework::Soc2, Framework::Soc2];
        assert_eq!(
            request.validate(),
            Err(ValidationError::DuplicateFramework(Framework::Soc2))
        );
    }

    #[test]
    fn combines_markdown_and_database_context() {
        let context = AnalysisContext {
            markdown: "## Product\nA hosted control plane.".into(),
            database_record_json: r#"{"tenant_id":"tenant-1","risk":"moderate"}"#.into(),
        };
        let rendered = context.render(4096).expect("context should fit");
        assert!(rendered.contains("Reviewed Markdown Context"));
        assert!(rendered.contains("Owner-Scoped Database Context"));
        assert!(rendered.contains("tenant-1"));
    }

    #[test]
    fn rejects_oversized_context() {
        let context = AnalysisContext {
            markdown: "a".repeat(128),
            database_record_json: "{}".into(),
        };
        assert!(matches!(
            context.render(16),
            Err(ValidationError::ContextTooLarge { .. })
        ));
    }
}
