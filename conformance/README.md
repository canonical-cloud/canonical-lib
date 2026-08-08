# Conformance corpus — the behavioral contract

`canonical-lib` will ship three peer implementations: Rust, Dart, and TypeScript.
No compiler can check that they agree, so **this corpus is the only mechanism
preventing drift.** It is the specification, not a description of any one
implementation.

```sh
node conformance/build.mjs           # regenerate cases/corpus.json from cases.mjs
node conformance/build.mjs --check   # CI: fail if the committed corpus is stale or invalid
```

## Spec-first, and what that means here

No language is normative — that is the point of choosing peers. So the corpus is
**authored and reviewed as the spec**, never regenerated from an implementation.
Regenerating from Rust would silently make Rust the source of truth and demote
the other two, which is precisely the design that was rejected.

A rule change goes:

1. amend `cases.mjs` — reviewed on its own merits, as a spec change;
2. all three implementations go red;
3. each is updated;
4. green means all three agree.

A rule that cannot be expressed as a corpus case is a rule that cannot be
verified across three languages.

### Why `cases.mjs` exists rather than hand-written JSON

Hand-authoring exact JSON strings is the known weak point of a spec-first
corpus: a typo in an expected byte sequence produces a confidently wrong
contract. `cases.mjs` holds the cases as readable structures and `build.mjs`
encodes them, so the reviewable artifact is legible and the committed artifact is
byte-exact.

This is **not** "generated from an implementation" — the generator is the spec
source itself, and it has no knowledge of Rust, Dart, or TypeScript.

## Envelope

A request is a JSON object with an `op` discriminator; the response is a JSON
object. Both cross the boundary as **strings**, and the contract is over the
exact bytes — an implementation that reorders keys or drops a null while
re-serializing is caught rather than excused.

```jsonc
// request
{"op":"validate_quote_request","request":{ …QuoteRequest… }}

// accepted
{"status":"ok"}

// rejected
{"status":"rejected","errors":[{"field":"organizationName","kind":"missing"}]}
```

Key order is part of the contract: `status` then `errors`; within an error,
`field`, `kind`, then any numeric parameters in the order listed below. Rust
(`serde` struct order), Dart (map literal order), and TypeScript (object literal
order) all preserve insertion order, so this is satisfiable in all three without
a custom serializer.

## Decision: collect-all, one error per field

**All failing fields are reported, not just the first.**

The existing Rust `validate_wire_quote` returns on first failure. That is being
changed deliberately, and it is worth being explicit about why: this validates a
customer-facing intake form. First-failure forces someone with four problems
through four submit cycles. Fixing this now, while there is one implementation,
costs far less than changing the contract later across three languages and their
consumers.

**A field yields at most one error** — the first rule it fails, in the rule order
documented per field below. Without that, "how many errors does one bad field
produce" becomes a fourth thing three implementations must agree on.

**Errors are ordered by schema property declaration order** — the order of
`$defs.QuoteRequest.properties` in `canonical-interfaces/schema/quote.schema.json`:

```
organizationName, contactName, contactEmail, website, employeeCount,
annualRevenueBand, frameworks, currentStage, infrastructure, dataSensitivity,
targetDate, hasSecurityProgram, hasPolicies, hasRiskAssessment,
hasIncidentResponsePlan, hasVendorManagement, notes, contextKey, answersVersion
```

That ordering is language-neutral, already matches the Rust validator's order,
and every generator preserves property order, so no implementation has to encode
a separate list.

## Decision: errors never echo a value

`wire.rs` already establishes this — *"Errors identify only the field, never its
value"* — and the corpus makes it enforceable. An error carries a field name, a
kind, and numeric bounds. It never carries the offending value, because a quote
request contains customer identity and infrastructure detail that must not end up
in a client log or an error-reporting pipeline.

Numeric parameters are lengths and limits, never content. `{"kind":"too_long",
"limit":200,"actual":247}` is safe; the 247 characters are not.

The corpus includes a case that submits a recognizable secret-shaped string in a
free-text field and asserts it does not appear anywhere in the response.

## Decision: messages are NOT contractual

Only `field`, `kind`, and numeric parameters are. Human-readable text is
deliberately excluded so each implementation can produce idiomatic messages and
so the product can localize without a breaking change to a cross-language wire
contract.

## Error vocabulary

| `kind` | Meaning | Parameters |
| --- | --- | --- |
| `missing` | required value absent, or empty after trimming | — |
| `not_trimmed` | leading or trailing whitespace | — |
| `too_long` | exceeds the maximum character count | `limit`, `actual` |
| `too_few` | array below `minItems` | `limit`, `actual` |
| `too_many` | array above `maxItems` | `limit`, `actual` |
| `duplicate` | repeated item in a set-valued field | — |
| `not_allowed` | value outside the permitted set | — |
| `malformed` | fails a format rule (email, URL, date, key) | — |
| `out_of_range` | numeric value outside its bounds | `min`, `max` |
| `unsupported_version` | `answersVersion` is not 1 | `expected` |

`not_allowed` deliberately does **not** list the permitted values. The allowed
sets live in `canonical-interfaces` and a client that needs them should read them
from there rather than scraping an error payload.

## Per-field rules, in evaluation order

Each field stops at its first failure.

| Field | Rules, in order |
| --- | --- |
| `organizationName` | `missing` (empty after trim) → `not_trimmed` → `too_long` (200) |
| `contactName` | `missing` → `not_trimmed` → `too_long` (160) |
| `contactEmail` | `missing` → `not_trimmed` → `too_long` (320) → `malformed` |
| `website` *(optional)* | `too_long` (2048) → `malformed` (must be `http(s)://host.tld`) |
| `employeeCount` | `out_of_range` (1…1 000 000) |
| `annualRevenueBand` *(optional)* | `not_allowed` |
| `frameworks` | `too_few` (1) → `too_many` (12) → `not_allowed` → `duplicate` |
| `currentStage` | `not_allowed` |
| `infrastructure` | `too_few` (1) → `too_many` (12) → `not_allowed` → `duplicate` |
| `dataSensitivity` | `too_few` (1) → `too_many` (12) → `not_allowed` → `duplicate` |
| `targetDate` *(optional)* | `malformed` (`YYYY-MM-DD`, month 1–12, day 1–31) |
| `notes` *(optional)* | `too_long` (5000) |
| `contextKey` *(optional)* | `malformed` (ASCII, first char `[a-z0-9]`, rest `[a-z0-9._-]`, ≤128) |
| `answersVersion` | `unsupported_version` (must be 1) |

Boolean fields carry no rules; they are validated by the type system in every
target language.

Lengths are counted in **Unicode scalar values**, not UTF-8 bytes or UTF-16 code
units. This matters: Rust `chars().count()`, Dart `runes.length`, and JavaScript
`[...s].length` agree, while Dart's `String.length` and JavaScript's `.length`
would both disagree on any astral character. The corpus includes an emoji case
that fails if an implementation uses the naive length.

`contextKey` is the one exception — it is constrained to ASCII, so its ≤128 limit
is unambiguous.

## Known delta from the current implementation

`src/wire.rs` today is first-failure and collapses everything to
`InvalidField(field)` / `DuplicateItem(field)`. Bringing it to this contract is
tracked as #14, and is expected to be the largest single change of the three
implementations precisely because it already exists.

Do **not** resolve a disagreement between this corpus and `wire.rs` by editing
the corpus to match. The corpus is the spec; if it is wrong, it is changed as a
reviewed spec change, not silently reconciled toward whichever implementation
happens to exist.
