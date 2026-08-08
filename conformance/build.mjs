#!/usr/bin/env node
// Encode conformance/cases.mjs into cases/corpus.json, validating the spec on
// the way through.
//
//   node conformance/build.mjs           # write cases/corpus.json
//   node conformance/build.mjs --check   # CI: fail if stale or invalid
//
// The checks here exist because hand-authored expected output is the known weak
// point of a spec-first corpus. Everything that can be derived is derived and
// compared against what the author wrote, so a typo fails the build instead of
// becoming a confidently wrong contract.

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { CASES, VALID, CANARIES } from "./cases.mjs";

const here = path.dirname(fileURLToPath(import.meta.url));
const outPath = path.join(here, "cases", "corpus.json");

// Property declaration order in canonical-interfaces defines error ordering.
// Read from the dependency rather than restated, so the two cannot drift.
const SCHEMA_CANDIDATES = [
  process.env.CANONICAL_INTERFACES_DIR,
  path.resolve(here, "..", "..", "canonical-interfaces"),
].filter(Boolean);

function loadFieldOrder() {
  for (const dir of SCHEMA_CANDIDATES) {
    const p = path.join(dir, "schema", "quote.schema.json");
    if (!fs.existsSync(p)) continue;
    const defs = JSON.parse(fs.readFileSync(p, "utf8")).$defs.QuoteRequest;
    return { order: Object.keys(defs.properties), source: p };
  }
  return { order: null, source: null };
}

const VOCABULARY = {
  missing: [],
  not_trimmed: [],
  too_long: ["limit", "actual"],
  too_few: ["limit", "actual"],
  too_many: ["limit", "actual"],
  duplicate: [],
  not_allowed: [],
  malformed: [],
  out_of_range: ["min", "max"],
  unsupported_version: ["expected"],
};

const scalarLength = (s) => [...s].length;

const problems = [];
const fail = (caseName, msg) => problems.push(`${caseName}: ${msg}`);

const { order: FIELD_ORDER, source: schemaSource } = loadFieldOrder();
if (!FIELD_ORDER) {
  console.error(
    "error: canonical-interfaces checkout not found; set CANONICAL_INTERFACES_DIR.\n" +
    "       Field order and field names are read from its schema so this corpus\n" +
    "       cannot drift from the contract it encodes.",
  );
  process.exit(2);
}
const rank = new Map(FIELD_ORDER.map((f, i) => [f, i]));

function buildRequest(patch) {
  const merged = { ...VALID, ...patch };
  // `undefined` in a patch means "omit this optional field entirely", which is
  // distinct from null and must not appear as a key in the encoded request.
  const out = {};
  for (const key of FIELD_ORDER) {
    if (merged[key] !== undefined) out[key] = merged[key];
  }
  for (const key of Object.keys(merged)) {
    if (!rank.has(key)) out[key] = merged[key]; // surfaced as a problem below
  }
  return out;
}

const cases = [];
const seenNames = new Set();

for (const c of CASES) {
  if (seenNames.has(c.name)) fail(c.name, "duplicate case name");
  seenNames.add(c.name);

  for (const key of Object.keys(c.patch)) {
    if (!rank.has(key)) fail(c.name, `patches unknown field "${key}"`);
  }

  const request = buildRequest(c.patch);

  for (const err of c.errors) {
    if (!rank.has(err.field)) {
      fail(c.name, `error names unknown field "${err.field}"`);
      continue;
    }
    const params = VOCABULARY[err.kind];
    if (!params) {
      fail(c.name, `unknown kind "${err.kind}" (not in the documented vocabulary)`);
      continue;
    }
    const got = Object.keys(err).filter((k) => k !== "field" && k !== "kind");
    if (got.join(",") !== params.join(",")) {
      fail(c.name, `kind "${err.kind}" expects params [${params}] in that order, got [${got}]`);
    }
    // Re-derive what can be derived. This is the check that catches arithmetic
    // slips in hand-authored expectations.
    if (err.kind === "too_long" && typeof request[err.field] === "string") {
      const actual = scalarLength(request[err.field]);
      if (actual !== err.actual) {
        fail(c.name, `too_long actual=${err.actual} but the value is ${actual} scalar values`);
      }
      if (actual <= err.limit) {
        fail(c.name, `too_long but ${actual} does not exceed limit ${err.limit}`);
      }
    }
    if ((err.kind === "too_few" || err.kind === "too_many") && Array.isArray(request[err.field])) {
      const actual = request[err.field].length;
      if (actual !== err.actual) {
        fail(c.name, `${err.kind} actual=${err.actual} but the array holds ${actual}`);
      }
    }
  }

  const ranks = c.errors.map((e) => rank.get(e.field));
  const sorted = [...ranks].sort((a, b) => a - b);
  if (ranks.join(",") !== sorted.join(",")) {
    fail(c.name, `errors are not in schema property order: ${c.errors.map((e) => e.field).join(", ")}`);
  }
  if (new Set(c.errors.map((e) => e.field)).size !== c.errors.length) {
    fail(c.name, "more than one error for a single field (the contract allows exactly one)");
  }

  const response = c.errors.length === 0
    ? { status: "ok" }
    : { status: "rejected", errors: c.errors };

  const encodedResponse = JSON.stringify(response);
  for (const canary of CANARIES) {
    if (encodedResponse.includes(canary)) {
      fail(c.name, `response echoes a submitted value (found "${canary}")`);
    }
  }

  cases.push({
    name: c.name,
    request: JSON.stringify({ op: "validate_quote_request", request }),
    response: encodedResponse,
  });
}

if (problems.length) {
  console.error(`error: ${problems.length} problem(s) in the corpus spec:`);
  for (const p of problems) console.error(`  - ${p}`);
  process.exit(2);
}

const corpus = {
  $comment: "Generated from conformance/cases.mjs by conformance/build.mjs. Edit cases.mjs, not this file.",
  version: 1,
  // A logical identifier, deliberately not a filesystem path: the schema is read
  // from a sibling checkout locally and a temp clone in CI, and embedding either
  // would make the artifact non-reproducible and permanently "stale".
  fieldOrderSource: "canonical-cloud/canonical-interfaces schema/quote.schema.json#$defs.QuoteRequest.properties",
  caseCount: cases.length,
  cases,
};
const content = JSON.stringify(corpus, null, 2) + "\n";

if (process.argv.includes("--check")) {
  const current = fs.existsSync(outPath) ? fs.readFileSync(outPath, "utf8") : null;
  if (current !== content) {
    console.error("error: cases/corpus.json is stale — run: node conformance/build.mjs");
    process.exit(1);
  }
  console.log(`corpus up to date (${cases.length} cases)`);
} else {
  fs.mkdirSync(path.dirname(outPath), { recursive: true });
  fs.writeFileSync(outPath, content);
  console.log(`wrote ${path.relative(process.cwd(), outPath)} (${cases.length} cases)`);
}
