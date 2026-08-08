// The conformance corpus, authored as the specification.
//
// This file is the reviewable artifact; build.mjs encodes it into
// cases/corpus.json with exact-byte request/response strings. It has no
// knowledge of Rust, Dart, or TypeScript — the corpus is spec-first, and
// regenerating it from an implementation would silently make that
// implementation normative. See README.md.

/** A request that passes every rule. Cases override single fields from it. */
export const VALID = {
  organizationName: "Northwind Traders",
  contactName: "Ada Lovelace",
  contactEmail: "ada@northwind.example",
  website: "https://northwind.example",
  employeeCount: 42,
  annualRevenueBand: "1m_10m",
  frameworks: ["soc2_type_2"],
  currentStage: "readiness",
  infrastructure: ["aws"],
  dataSensitivity: ["pii"],
  targetDate: "2027-03-01",
  hasSecurityProgram: true,
  hasPolicies: true,
  hasRiskAssessment: false,
  hasIncidentResponsePlan: false,
  hasVendorManagement: true,
  notes: "Renewal in Q1.",
  contextKey: "quote-analysis",
  answersVersion: 1,
};

const ok = (name, patch = {}) => ({ name, patch, errors: [] });
const bad = (name, patch, ...errors) => ({ name, patch, errors });
const e = (field, kind, params = {}) => ({ field, kind, ...params });

// A string of `n` scalar values built from a 2-UTF-16-unit emoji. An
// implementation using Dart's String.length or JavaScript's .length counts
// double and will report the wrong `actual`, or pass when it should fail.
const emoji = (n) => "\u{1F510}".repeat(n);

export const CASES = [
  // --- accepted -------------------------------------------------------------
  ok("valid/complete"),
  ok("valid/optionals-absent", {
    website: undefined,
    annualRevenueBand: undefined,
    targetDate: undefined,
    notes: undefined,
    contextKey: undefined,
  }),
  ok("valid/boundary-max-lengths", {
    organizationName: "o".repeat(200),
    contactName: "c".repeat(160),
    notes: "n".repeat(5000),
  }),
  ok("valid/boundary-employee-count-min", { employeeCount: 1 }),
  ok("valid/boundary-employee-count-max", { employeeCount: 1_000_000 }),
  ok("valid/boundary-twelve-frameworks", {
    frameworks: [
      "soc2_type_1", "soc2_type_2", "nist_csf_2", "nist_800_53", "hipaa",
      "iso_27001", "pci_dss_4", "fedramp", "gdpr", "custom",
    ],
  }),
  ok("valid/unicode-name-within-limit", { organizationName: emoji(200) }),
  ok("valid/context-key-charset", { contextKey: "a0.b_c-d" }),

  // --- organizationName -----------------------------------------------------
  bad("organizationName/empty", { organizationName: "" }, e("organizationName", "missing")),
  bad("organizationName/whitespace-only", { organizationName: "   " }, e("organizationName", "missing")),
  bad("organizationName/untrimmed", { organizationName: " Northwind " }, e("organizationName", "not_trimmed")),
  bad("organizationName/too-long", { organizationName: "o".repeat(201) },
    e("organizationName", "too_long", { limit: 200, actual: 201 })),
  // Counted in scalar values: 201 emoji is 402 UTF-16 units and 804 bytes.
  bad("organizationName/too-long-unicode", { organizationName: emoji(201) },
    e("organizationName", "too_long", { limit: 200, actual: 201 })),

  // --- contactName ----------------------------------------------------------
  bad("contactName/empty", { contactName: "" }, e("contactName", "missing")),
  bad("contactName/too-long", { contactName: "c".repeat(161) },
    e("contactName", "too_long", { limit: 160, actual: 161 })),

  // --- contactEmail ---------------------------------------------------------
  bad("contactEmail/empty", { contactEmail: "" }, e("contactEmail", "missing")),
  bad("contactEmail/untrimmed", { contactEmail: " ada@northwind.example" }, e("contactEmail", "not_trimmed")),
  bad("contactEmail/no-at", { contactEmail: "ada.northwind.example" }, e("contactEmail", "malformed")),
  bad("contactEmail/two-at", { contactEmail: "ada@a@northwind.example" }, e("contactEmail", "malformed")),
  bad("contactEmail/empty-local", { contactEmail: "@northwind.example" }, e("contactEmail", "malformed")),
  bad("contactEmail/domain-without-dot", { contactEmail: "ada@northwind" }, e("contactEmail", "malformed")),
  bad("contactEmail/inner-whitespace", { contactEmail: "ada l@northwind.example" }, e("contactEmail", "malformed")),
  // 311 + "@x.example" (10) = 321, one past the limit.
  bad("contactEmail/too-long", { contactEmail: `${"a".repeat(311)}@x.example` },
    e("contactEmail", "too_long", { limit: 320, actual: 321 })),

  // --- website (optional) ---------------------------------------------------
  bad("website/no-scheme", { website: "northwind.example" }, e("website", "malformed")),
  bad("website/host-without-dot", { website: "https://localhost" }, e("website", "malformed")),
  bad("website/empty-host", { website: "https://" }, e("website", "malformed")),
  bad("website/credentials-in-host", { website: "https://user@northwind.example" }, e("website", "malformed")),
  bad("website/too-long", { website: `https://x.example/${"p".repeat(2040)}` },
    e("website", "too_long", { limit: 2048, actual: 2058 })),

  // --- employeeCount --------------------------------------------------------
  bad("employeeCount/zero", { employeeCount: 0 },
    e("employeeCount", "out_of_range", { min: 1, max: 1_000_000 })),
  bad("employeeCount/negative", { employeeCount: -1 },
    e("employeeCount", "out_of_range", { min: 1, max: 1_000_000 })),
  bad("employeeCount/above-max", { employeeCount: 1_000_001 },
    e("employeeCount", "out_of_range", { min: 1, max: 1_000_000 })),

  // --- annualRevenueBand (optional) ----------------------------------------
  bad("annualRevenueBand/unknown", { annualRevenueBand: "series_z" },
    e("annualRevenueBand", "not_allowed")),

  // --- frameworks -----------------------------------------------------------
  bad("frameworks/empty", { frameworks: [] }, e("frameworks", "too_few", { limit: 1, actual: 0 })),
  bad("frameworks/unknown-value", { frameworks: ["soc2_type_3"] }, e("frameworks", "not_allowed")),
  bad("frameworks/duplicate", { frameworks: ["hipaa", "hipaa"] }, e("frameworks", "duplicate")),
  // not_allowed precedes duplicate: the unknown value is reached first.
  bad("frameworks/unknown-before-duplicate", { frameworks: ["nope", "hipaa", "hipaa"] },
    e("frameworks", "not_allowed")),

  // --- currentStage ---------------------------------------------------------
  bad("currentStage/unknown", { currentStage: "shipping" }, e("currentStage", "not_allowed")),

  // --- infrastructure / dataSensitivity -------------------------------------
  bad("infrastructure/empty", { infrastructure: [] }, e("infrastructure", "too_few", { limit: 1, actual: 0 })),
  bad("infrastructure/duplicate", { infrastructure: ["aws", "aws"] }, e("infrastructure", "duplicate")),
  bad("dataSensitivity/empty", { dataSensitivity: [] }, e("dataSensitivity", "too_few", { limit: 1, actual: 0 })),
  bad("dataSensitivity/unknown", { dataSensitivity: ["top_secret"] }, e("dataSensitivity", "not_allowed")),

  // --- targetDate (optional) ------------------------------------------------
  bad("targetDate/wrong-shape", { targetDate: "03/01/2027" }, e("targetDate", "malformed")),
  bad("targetDate/month-zero", { targetDate: "2027-00-01" }, e("targetDate", "malformed")),
  bad("targetDate/month-thirteen", { targetDate: "2027-13-01" }, e("targetDate", "malformed")),
  bad("targetDate/day-zero", { targetDate: "2027-03-00" }, e("targetDate", "malformed")),
  bad("targetDate/day-thirty-two", { targetDate: "2027-03-32" }, e("targetDate", "malformed")),
  bad("targetDate/non-numeric", { targetDate: "2027-0a-01" }, e("targetDate", "malformed")),

  // --- notes (optional) -----------------------------------------------------
  bad("notes/too-long", { notes: "n".repeat(5001) },
    e("notes", "too_long", { limit: 5000, actual: 5001 })),

  // --- contextKey (optional) ------------------------------------------------
  bad("contextKey/empty", { contextKey: "" }, e("contextKey", "malformed")),
  bad("contextKey/uppercase", { contextKey: "Quote" }, e("contextKey", "malformed")),
  bad("contextKey/leading-dash", { contextKey: "-quote" }, e("contextKey", "malformed")),
  bad("contextKey/space", { contextKey: "quote analysis" }, e("contextKey", "malformed")),
  bad("contextKey/non-ascii", { contextKey: "quoté" }, e("contextKey", "malformed")),
  bad("contextKey/too-long", { contextKey: "q".repeat(129) }, e("contextKey", "malformed")),

  // --- answersVersion -------------------------------------------------------
  bad("answersVersion/zero", { answersVersion: 0 },
    e("answersVersion", "unsupported_version", { expected: 1 })),
  bad("answersVersion/future", { answersVersion: 2 },
    e("answersVersion", "unsupported_version", { expected: 1 })),

  // --- collect-all and ordering ---------------------------------------------
  // The decision that separates this contract from the current first-failure
  // implementation: every failing field is reported, in schema property order.
  bad("multi/all-fields-reported",
    { organizationName: "", contactEmail: "nope", employeeCount: 0, frameworks: [], answersVersion: 7 },
    e("organizationName", "missing"),
    e("contactEmail", "malformed"),
    e("employeeCount", "out_of_range", { min: 1, max: 1_000_000 }),
    e("frameworks", "too_few", { limit: 1, actual: 0 }),
    e("answersVersion", "unsupported_version", { expected: 1 })),
  // Ordering follows schema declaration order, not the order fields were
  // corrupted, and not alphabetical -- contextKey precedes answersVersion.
  bad("multi/ordering-is-schema-order",
    { answersVersion: 3, contextKey: "BAD", organizationName: "  x  " },
    e("organizationName", "not_trimmed"),
    e("contextKey", "malformed"),
    e("answersVersion", "unsupported_version", { expected: 1 })),
  // One field, several broken rules -> exactly one error, the first rule.
  bad("multi/one-error-per-field", { organizationName: ` ${"o".repeat(250)} ` },
    e("organizationName", "not_trimmed")),

  // --- privacy --------------------------------------------------------------
  // Errors carry field, kind, and numeric bounds only. build.mjs asserts this
  // marker appears nowhere in the encoded response.
  bad("privacy/no-value-echo",
    { notes: `SECRET-CANARY-${"z".repeat(5000)}`, contextKey: "SECRET-CANARY-KEY" },
    e("notes", "too_long", { limit: 5000, actual: 5014 }),
    e("contextKey", "malformed")),
];

/** Strings that must never appear in any encoded response. */
export const CANARIES = ["SECRET-CANARY"];
