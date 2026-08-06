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
owns domain validation and deterministic context assembly. Applications adapt
wire payloads into these types rather than duplicating business rules.

The dependency is declared in `.zpkg.toml`; Zed package resolution is the
organization-level source dependency contract. This crate deliberately remains
transport-agnostic so it can be reused by REST, WebSocket, CLI, Flutter, and MCP
consumers.

## Included foundation

- bounded quote-intake validation for SOC 2, NIST, HIPAA, ISO 27001, and PCI DSS;
- deterministic combination of a reviewed Markdown context file with one
  owner-scoped `canonical_context` PostgreSQL record serialized as JSON;
- explicit size limits before model input or logging;
- zero network, database, authentication, or model-provider dependencies.

## Develop

```sh
cargo fmt --all -- --check
cargo test --all-targets --locked
cargo clippy --all-targets --locked -- -D warnings
```

Never put credentials, unredacted regulated data, or cross-tenant records into
an `AnalysisContext`. The API layer owns authorization and owner scoping before
constructing this domain object.
