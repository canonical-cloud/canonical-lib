import assert from 'node:assert/strict';

import { analyzeSource, formatReport } from './require-send.mjs';

function findings(language, source) {
  return analyzeSource(source, language);
}

const cases = [
  {
    name: 'Rust reports an undelivered chain',
    language: 'rust',
    source: 'fn run(logger: Logger) { logger.info("ready"); }',
    count: 1,
  },
  {
    name: 'Rust accepts a delivered chain',
    language: 'rust',
    source: 'fn run(logger: Logger) { logger.info("ready").send(); }',
    count: 0,
  },
  {
    name: 'Rust accepts a chain returned to its caller',
    language: 'rust',
    source: 'fn event(logger: Logger) -> Event { logger.info("ready") }',
    count: 0,
  },
  {
    name: 'Rust tracks an assigned event until delivery',
    language: 'rust',
    source: 'fn run(logger: Logger) { let event = logger.warn("slow"); event.send(); }',
    count: 0,
  },
  {
    name: 'Rust reports an assigned event abandoned at scope exit',
    language: 'rust',
    source: 'fn run(logger: Logger) { let event = logger.warn("slow"); }',
    count: 1,
  },
  {
    name: 'Dart accepts send with an explicit delivery flag',
    language: 'dart',
    source: 'void run(Logger logger) { logger.error("failed").send(true); }',
    count: 0,
  },
  {
    name: 'Gleam reports an undelivered pipeline',
    language: 'gleam',
    source: 'pub fn run(logger) { let event = logging.info(logger, "ready") |> logging.context([]) Nil }',
    count: 1,
  },
  {
    name: 'Gleam accepts a delivered pipeline',
    language: 'gleam',
    source: 'pub fn run(logger) { logging.info(logger, "ready") |> logging.send }',
    count: 0,
  },
  {
    name: 'Convenience logger calls are already terminal',
    language: 'rust',
    source: 'fn run(logger: Logger) { logger.log(level, "ready", context, fields); }',
    count: 0,
  },
  {
    name: 'Comments and strings cannot fabricate chains',
    language: 'rust',
    source: 'fn run() { // logger.info("ignored");\n let text = "logger.error(ignored)"; }',
    count: 0,
  },
  {
    name: 'The next-line suppression is honored',
    language: 'rust',
    source: '// ores-lint-disable-next-line require-send\nlogger.info("intentional");',
    count: 0,
  },
  {
    name: 'The file suppression is honored',
    language: 'dart',
    source: '// ores-lint-disable-file require-send\nlogger.warn("intentional");',
    count: 0,
  },
];

for (const testCase of cases) {
  const actual = findings(testCase.language, testCase.source);
  assert.equal(
    actual.length,
    testCase.count,
    `${testCase.name}: expected ${testCase.count}, received ${JSON.stringify(actual)}`,
  );
}

assert.match(
  formatReport([{ file: 'src/main.rs', findings: findings('rust', 'logger.info("ready");') }]),
  /src\/main\.rs:1:1/,
);
assert.match(
  formatReport([{ file: 'src/main.rs', findings: [] }]),
  /clean \(1 file scanned\)/,
);

process.stdout.write(`require-send scanner fixtures passed (${cases.length} cases)\n`);
