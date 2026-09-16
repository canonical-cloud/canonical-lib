# canonical-lib — deprecated

> **Deprecated / frozen.** The maintained implementation has been consolidated into the successor code repository that is being finalized as `canonical-lib-code`. The current pre-rename repository identity is `canonical-lib-core`.

Do not add new features, fixes, contracts, or integrations to this repository.

## Migration

- private shared implementation code: `canonical-lib-code` (currently `canonical-lib-core` until the GitHub rename is completed)
- client-safe publishable shared code: `canonical-pub-lib-core`
- database/ORM implementation and private persistence helpers: `canonical-orm-core`
- generated contracts and RPC type/path authority: `canonical-interfaces`

The Rust crate name remains `canonical-lib` for import compatibility; the repository-name change does not require consumers to rename the crate immediately.

## Status

The legacy shared implementation represented here has already been folded into the successor code line. This repository is retained only for provenance, historical commits, and migration traceability and should be archived once the repository-name cutover is complete.

No credentials, customer data, or new production dependencies should be introduced here.
