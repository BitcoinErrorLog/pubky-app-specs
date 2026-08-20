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
- **Version**: `0.6.2-marketplace.1` (crate and npm package). The pre-release
  suffix makes it unambiguous that this is a fork build derived from 0.6.2.
  Subsequent fork builds increment the final number (`-marketplace.2`, ...).
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
npm pack                           # produces pubky-app-specs-0.6.2-marketplace.1.tgz
```

## How to consume (no npm publish required)

The tarball is attached to the GitHub release
[`v0.6.2-marketplace.1`](https://github.com/BitcoinErrorLog/pubky-app-specs/releases/tag/v0.6.2-marketplace.1).
Point the app's dependency at it directly:

```json
"pubky-app-specs": "https://github.com/BitcoinErrorLog/pubky-app-specs/releases/download/v0.6.2-marketplace.1/pubky-app-specs-0.6.2-marketplace.1.tgz"
```
