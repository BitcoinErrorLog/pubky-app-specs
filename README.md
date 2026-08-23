# pubky-app-specs

[![crates.io](https://img.shields.io/crates/v/pubky-app-specs)](https://crates.io/crates/pubky-app-specs)
[![docs.rs](https://img.shields.io/docsrs/pubky-app-specs)](https://docs.rs/pubky-app-specs)
[![npm](https://img.shields.io/npm/v/pubky-app-specs)](https://www.npmjs.com/package/pubky-app-specs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

Rust types, sanitization, and validation for [Pubky.app](https://pubky.app) data models. Use this crate to build JSON that matches what [Pubky indexers](https://github.com/pubky/pubky-nexus) expect.

## This fork: Pubky Marketplace project

This is `BitcoinErrorLog/pubky-app-specs`, a fork of the official
[`pubky/pubky-app-specs`](https://github.com/pubky/pubky-app-specs) adding
the **marketplace protocol objects** (versions `0.6.2-marketplace.N`; the
npm package name is unchanged). The objects are deliberately fork-only for
now — official specs parse these paths as `Resource::Unknown` — and no
upstream PRs are filed while the protocol shape settles.

**Added over upstream** (all closed-world camelCase records with full
validation, wasm builders, and tests):

- Public records under `/pub/pubky.app/marketplace/v1/…`: **shop** (with
  the optional `transactionService` authority declaration), **listing**
  (variants, shipping, auction terms, digital locks, taxonomy-bounded
  attributes, integer minor-unit money), **review** + **review-response**,
  and **drop** (timed limited releases, ADR 0026).
- The first `/priv/` records: the cross-device **watchlist** document and
  the portable **order receipt** (with optional drop-edition fields) —
  private, never indexed, deliberately outside the URI parser.
- Offline-verifiable compact-JWS attestations with normative verification
  recipes: `pubky-purchase-attestation+v1` (review eligibility),
  `pubky-order-receipt+v1` (portable receipts), and
  `pubky-drop-edition+v1` (edition numbers). Signatures prove key
  possession; trust in `iss` is explicitly the verifier's policy.
- Collection posts accept canonical listing URIs.

**Fixes:** the listing path-ID rule rejected real UUID-keyed listings until
`.4`→`.5`; corrected so deployed records parse.

Full changelog and consumption notes: [`MARKETPLACE-FORK.md`](MARKETPLACE-FORK.md);
normative field tables: [`SPEC.md`](SPEC.md).

> ⚠️ **Warning: Rapid Development Phase**  
> This specification is in an **early development phase** and is evolving quickly. Expect frequent changes and updates as the system matures. Consider this a **v0 draft**.
>
> When we reach the first stable, long-term support version of the schemas, paths will adopt the format: `pubky.app/v1/` to indicate compatibility and stability

## Installation

**Rust** ([crates.io](https://crates.io/crates/pubky-app-specs)):

```bash
cargo add pubky-app-specs
```

**JavaScript / TypeScript** ([npm](https://www.npmjs.com/package/pubky-app-specs)): see [`pkg/README.md`](https://github.com/pubky/pubky-app-specs/blob/main/pkg/README.md).

## Rust quick start

```rust
use pubky_app_specs::{
    traits::{HasPath, Validatable},
    PubkyAppUser,
};
use serde_json::to_vec;

// Create a user profile
let user = PubkyAppUser::new("Alice".into(), None, None, None, None);
let path = PubkyAppUser::create_path(); // /pub/pubky.app/profile.json
let json = to_vec(&user).unwrap();

// Parse and validate JSON from storage
let profile = PubkyAppUser::try_from(&json, "").unwrap();
```

For a full homeserver flow, see [`examples/create_user.rs`](https://github.com/pubky/pubky-app-specs/blob/main/examples/create_user.rs).

## Why use this crate

- **Validation consistency** — same sanitization and validation rules as Pubky indexers.
- **Auto IDs and paths** — generates IDs, paths, and URLs according to Pubky standards.
- **Single source of truth** — Rust models drive native apps, WASM bindings, and this spec.

## Features

| Feature   | Purpose                        |
| --------- | ------------------------------ |
| `openapi` | OpenAPI schemas via `utoipa`   |

```toml
pubky-app-specs = { version = "0.6", features = ["openapi"] }
```

- **MSRV:** 1.89 (see `rust-version` in `Cargo.toml`)
- **API docs:** [docs.rs/pubky-app-specs](https://docs.rs/pubky-app-specs)

## Models

| Rust type                   | Purpose                                  |
| --------------------------- | ---------------------------------------- |
| `PubkyAppUser`              | User profile information                 |
| `PubkyAppFile`              | Uploaded file metadata                   |
| `PubkyAppPost`              | Posts, replies, embeds, and collections  |
| `PubkyAppTag`               | Tags applied to Pubky URIs               |
| `PubkyAppBookmark`          | Bookmarks for Pubky URIs                 |
| `PubkyAppFollow`            | Follow relationships                     |
| `PubkyAppFeed`              | Feed configurations                      |
| `PubkyAppMute`              | Muted users                              |
| `PubkyAppBlob`              | Raw binary file data                     |
| `PubkyAppLastRead`          | Last-read notification timestamp         |
| `PubkyAppShop`              | Marketplace shop profile (singleton)     |
| `PubkyAppListing`           | Marketplace listing with variants        |
| `PubkyAppMarketplaceReview` | Marketplace review of a counterparty     |

## Specification

See the [full data model specification](https://github.com/pubky/pubky-app-specs/blob/main/SPEC.md) for URI layout, examples, and validation rules.

## License

MIT
