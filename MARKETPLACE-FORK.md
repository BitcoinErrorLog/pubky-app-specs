# Marketplace Fork Build (0.6.2 base)

This branch (`feat/marketplace-objects-0.6.x`) is a **fork build** maintained at
[BitcoinErrorLog/pubky-app-specs](https://github.com/BitcoinErrorLog/pubky-app-specs).
It is **not an upstream release** — the marketplace objects are pending upstream
review. Do not confuse it with an official `pubky/pubky-app-specs` version.

## Why this exists

pubky-app pins `pubky-app-specs` **0.6.2**. Upgrading the app to 0.8.0 breaks it
(`createFeed` changed to a single object argument and gained a required `icon`
field the app has no concept of). This branch adds the marketplace objects on
top of the exact 0.6.2 release so the app can adopt them with zero unrelated
breakage.

## Base and version scheme

- **Base commit**: `5caa830` — "chore: bump version to 0.6.2 (#148)", the
  upstream 0.6.2 release commit.
- **Version**: `0.6.2-marketplace.4` (crate and npm package). The pre-release
  suffix makes it unambiguous that this is a fork build derived from 0.6.2.
  Subsequent fork builds increment the final number (`-marketplace.5`, ...).
- The npm package **name** stays `pubky-app-specs` so app imports are unchanged.

## What was added

Cherry-picked from `feat/marketplace-objects` (identical file contents; the
marketplace sources are byte-for-byte the same on both branches):

- `src/models/marketplace.rs` — shared primitives: `PubkyAppMoney` (integer
  minor-unit amounts), `PubkyAppMarketplaceLocation`, RFC-3339 timestamp
  parsing, and common validators.
- `src/models/shop.rs` — `PubkyAppShop`, singleton at
  `/pub/pubky.app/marketplace/v1/shop.json`.
- `src/models/listing.rs` — `PubkyAppListing`, timestamp ID at
  `/pub/pubky.app/marketplace/v1/listings/{timestamp_id}`.
- `src/models/marketplace_review.rs` — `PubkyAppMarketplaceReview`, hash ID
  (hash of `{listing_uri}:{subject_pubky}:{role}`) at
  `/pub/pubky.app/marketplace/v1/reviews/{hash_id}`.
- Wiring into `PubkyAppObject`, the URI parser, URI builders, exports, wasm
  bindings (`createShop`, `createListing`, `createMarketplaceReview`), and
  docs (`SPEC.md`, `README.md`).

All records use camelCase strict (deny-unknown-fields) serialization and keep
every validation rule and test from the original branch.

## How to rebuild

```bash
git checkout feat/marketplace-objects-0.6.x
cargo fmt --check
cargo clippy --all-targets --all-features
cargo test
cargo run --bin bundle_specs_npm   # builds the npm package into pkg/
cd pkg && npm install && npm test
npm pack                           # produces pubky-app-specs-0.6.2-marketplace.4.tgz
```

Note: run the wasm/npm bundle from a worktree on the local disk — building
from the external volume trips over AppleDouble (`._*`) files.

## How to consume (no npm publish required)

The tarball is attached to the GitHub release
[`v0.6.2-marketplace.4`](https://github.com/BitcoinErrorLog/pubky-app-specs/releases/tag/v0.6.2-marketplace.4).
Point the app's dependency at it directly:

```json
"pubky-app-specs": "https://github.com/BitcoinErrorLog/pubky-app-specs/releases/download/v0.6.2-marketplace.4/pubky-app-specs-0.6.2-marketplace.4.tgz"
```

## Changes in `.4`

Category taxonomy support for the pubky-app marketplace taxonomy v2:

- **Listing `attributes` container** (`src/models/listing.rs`) — one stable,
  generic, bounded key/value container for item specifics:
  `attributes?: Record<string, string | string[]>` with at most 20 keys;
  keys are lowercase alphanumeric identifiers with single `-`/`_`
  separators (1–40 chars); values are trimmed strings of 1–80 chars; list
  values hold 1–10 unique entries. Serialized untagged (plain JSON strings
  or arrays). Exported as `PubkyAppListingAttributeValue`. Which keys a
  category expects (and their allowed values) is CLIENT configuration keyed
  by `taxonomyVersion` — the spec only enforces shape bounds, so the
  taxonomy can evolve without spec churn per category.
- **`taxonomyVersion` relaxed** from "must be 1" to a bounded integer
  (1–1,000,000), for the same reason: the category tree is versioned client
  config, not protocol schema.

## Changes in `.3`

Trust & reputation records (ADR 0024 in the pubky-app marketplace branch):

- **Purchase attestation format** (`src/models/marketplace_attestation.rs`) —
  the normative reference for the compact JWS (EdDSA/Ed25519) carried in a
  review record's `eligibilityAttestation`: closed-world `v: 1` claim set,
  structural parsing, offline signature verification (the `iss` pubky *is*
  the verification key), and review-binding checks. Exported as
  `PubkyAppPurchaseAttestation` / `PubkyAppPurchaseAttestationClaims`; wasm
  bindings `parsePurchaseAttestation` and `verifyPurchaseAttestation`.
- **`PubkyAppReviewResponse`** (`src/models/review_response.rs`) — the review
  subject's single revisable response at
  `/pub/pubky.app/marketplace/v1/review_responses/{review_id}` (path ID
  equals the subject review's ID). Structural subject-authorization helper
  `is_authorized_response_to`. URI parser variant
  `Resource::ReviewResponse`, builder `review_response_uri_builder`, wasm
  binding `createReviewResponse`.
- New dependency: `ed25519-dalek` (verification only, wasm-compatible).

## Changes in `.2`

- Collection `items` now also accept canonical marketplace listing URIs
  (`pubky://<pubky-id>/pub/pubky.app/marketplace/v1/listings/<listing-id>`)
  alongside canonical post URIs.
- Tests locking in that `PubkyAppTag` may target marketplace listing and shop
  URIs (already true — tag targets only require a valid URI — but now covered
  explicitly).
