# canonical-lib

Shared, transport-independent Rust domain logic for Canonical Cloud compliance
quotes.

## Dependency direction

```text
canonical-interfaces  →  canonical-lib  →  canonical-cli
                              ├──────────→ canonical-api-server.rs
                              ├──────────→ canonical-web-server.rs
                              └──────────→ canonical-mcp-server.rs
```

`canonical-interfaces` owns generated wire-format contracts. `canonical-lib`
owns domain validation and deterministic context assembly. Applications reuse
the generated Rust request through `canonical_lib::interfaces` and call
`validate_wire_quote` before adapting it into application-specific domain
objects.

The same dependency is declared in two complementary places:

- `.zpkg.toml` records the organization-level Zed package graph;
- `Cargo.toml` consumes the generated Rust crate from an immutable
  `canonical-interfaces` revision, recorded again in `Cargo.lock`.

This crate deliberately remains free of network, database, authentication, and
model-provider code so REST, WebSocket, CLI, Flutter, and MCP consumers can
share the same validation boundary.

## Included foundation

- native validation of the generated v1 compliance quote request;
- bounded quote-intake validation for SOC 2, NIST, HIPAA, ISO 27001, and PCI DSS;
- deterministic combination of a reviewed Markdown context file with one
  owner-scoped `canonical_context` PostgreSQL record serialized as JSON;
- explicit size limits before model input or logging;
- safe validation errors that identify fields without echoing customer input.

## Develop

```sh
cargo fmt --all -- --check
cargo test --all-targets --locked
cargo clippy --all-targets --locked -- -D warnings
```

Never put credentials, unredacted regulated data, or cross-tenant records into
an `AnalysisContext`. The API layer owns authorization and owner scoping before
constructing this domain object.
