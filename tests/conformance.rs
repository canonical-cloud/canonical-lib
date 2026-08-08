//! Replays `conformance/cases/corpus.json` through the dispatch entry point.
//!
//! The corpus is the specification, not a description of this crate. If a case
//! and this implementation disagree, the disagreement is resolved as a reviewed
//! spec change — never by editing the corpus to match the code, which would
//! quietly make Rust normative and defeat the point of three peer
//! implementations.

use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Deserialize)]
struct Corpus {
    #[serde(rename = "caseCount")]
    case_count: usize,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    name: String,
    request: String,
    response: String,
}

fn corpus() -> Corpus {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("conformance/cases/corpus.json");
    let raw = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
    serde_json::from_str(&raw).expect("corpus.json is not valid JSON")
}

#[test]
fn every_corpus_case_matches_byte_for_byte() {
    let corpus = corpus();
    assert_eq!(
        corpus.cases.len(),
        corpus.case_count,
        "corpus caseCount disagrees with the number of cases",
    );
    assert!(corpus.case_count > 0, "corpus is empty");

    let mut failures = Vec::new();
    for case in &corpus.cases {
        let actual = canonical_lib::dispatch(&case.request);
        if actual != case.response {
            failures.push(format!(
                "  {}\n    expected: {}\n    actual:   {}",
                case.name, case.response, actual,
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "{} of {} corpus cases disagree with this implementation:\n{}",
        failures.len(),
        corpus.case_count,
        failures.join("\n"),
    );
}

/// The corpus asserts this too, but it is cheap to re-assert against the real
/// serializer: a value that reaches an error payload is a privacy incident, not
/// a formatting bug.
#[test]
fn responses_never_echo_a_submitted_value() {
    for case in corpus().cases {
        let actual = canonical_lib::dispatch(&case.request);
        assert!(
            !actual.contains("SECRET-CANARY"),
            "{}: response echoed a submitted value: {actual}",
            case.name,
        );
    }
}

#[test]
fn unknown_ops_are_rejected_rather_than_panicking() {
    let response = canonical_lib::dispatch(r#"{"op":"drop_database","request":{}}"#);
    assert_eq!(
        response,
        r#"{"status":"rejected","errors":[{"field":"op","kind":"not_allowed"}]}"#,
    );
}

#[test]
fn malformed_envelopes_are_rejected_rather_than_panicking() {
    for input in ["", "{", "null", r#"{"op":"validate_quote_request"}"#] {
        let response = canonical_lib::dispatch(input);
        assert!(
            response.starts_with(r#"{"status":"rejected""#),
            "input {input:?} produced {response}",
        );
    }
}
