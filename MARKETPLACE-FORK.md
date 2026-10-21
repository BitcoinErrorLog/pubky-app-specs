# Marketplace Fork Build (0.6.2 base)

This branch (`marketplace-4-build`) is a **fork build** maintained at
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
- **Version**: `0.6.2-marketplace.15` (crate and npm package). The pre-release
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

All records use camelCase OPEN-WORLD serialization: unknown members are
tolerated on parse (aligned with the social/v1 forward-compat rule) so record
shapes can grow additively. The six JWS attestation header/claim structs
deliberately stay closed-world — an attestation is an opaque signed string,
not an evolving record. Every other validation rule and test from the
original branch is kept.

## How to rebuild

```bash
git checkout marketplace-4-build
cargo fmt --check
cargo clippy --all-targets --all-features
cargo test
cargo run --bin bundle_specs_npm   # builds the npm package into pkg/
cd pkg && npm install && npm test
npm pack                           # produces pubky-app-specs-0.6.2-marketplace.15.tgz
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

## Changes in `.15`

- **Optional review text.** `PubkyAppMarketplaceReview.text` may be empty
  (0–5,000 characters after trimming; a missing member reads as `""`).
  Records written under `.14` stay valid. Indexers must run `.15` before
  clients publish empty text, because `.14` rejects it.
- **Counting keys.** `item_count_key()` returns
  `(iss, order_ref, role, listing_owner, listing_id)`, so every item of a
  multi-item order counts on its own; `seller_rating_key()` returns
  `(iss, order_ref, role)`, the key under which `shipping` and
  `communication` count once per order. Both check that the attestation
  binds the record. A `.14` record keeps its key: it is the item review of
  the listing its attestation names.
- **Price weight.** New `marketplace_weight.rs`: `price_weight(currency,
  price_minor)` maps a price to a 1–5 weight by US-cent decade from a static
  table (`USD`, `EUR`, `GBP`, `CHF` offset 0; `SAT` −1). Unknown currencies
  weigh 1.
- **Wasm.** `itemCountKey(review)`, `sellerRatingKey(review)` (both run the
  full attestation check first) and `priceWeight(currency, priceMinor)`.

## Changes in `.14`

- **One review per attested order.** `PubkyAppMarketplaceReview` gains an
  optional `orderRef` (the embedded attestation's `order_ref` claim). When
  present it is part of the hash ID
  (`"{listing_uri}:{subject_pubky}:{role}:{orderRef}"`), so a repeat buyer's
  review of a second order gets its own path instead of overwriting the first.
  The attestation binding check requires `order_ref == orderRef`. Records
  without `orderRef` keep the three-part ID and stay valid, so nothing
  already published needs migrating.
- **One count per attested order.** A record with `orderRef` and a legacy one
  can embed the same attestation, and both verify at their own paths. A
  verified review therefore counts once per `(iss, order_ref)` and role: the
  record with `orderRef` counts, the legacy one is left out of counts,
  averages and lists. Helpers: `PubkyAppPurchaseAttestation::order_key()`
  returns `(iss, order_ref)`, `PubkyAppMarketplaceReview::is_order_scoped()`
  tells the two apart.

## Changes in `.13`

Reconciles the two fork lines. `marketplace-4-build` (`.9`-`.12`) and the
line Nexus pinned (`2f2797a8`, `cursor/specs-pin-7d79e5e8-22a0`) were rebased
from the same 0.6.2 base and diverged, so neither contained the other:

- From `marketplace-4-build`: open-world records, receipt attestation v2,
  entity listing ids in collections, and seller-set `deliveryAssumeDays` /
  `autoCompleteDays` on the shop record (PR #4).
- From the Nexus line, re-applied on top: `shipping` as a fulfillment method,
  `digitalLock` optional with `digital` fulfillment (still forbidden without
  it), and the `pubky-priv-aead/v1` specification text.

The listing test that expected an unknown lock field to be rejected now
asserts the open-world behavior: the field is tolerated and dropped.

## Changes in `.11`

Order receipt attestations now accept and verify the captured `pubky-order-receipt+v2` format with separate settlement and merchandise money objects, while preserving v1 parsing and verification. The wasm/JS parse and verify functions return snake_case claim keys for both versions.

## Changes in `.12`

Closed the v2 receipt-attestation validation and API-safety review items:
money validation now reuses `PubkyAppMoney`, record-value conversions fail
closed, v2 claims no longer expose a zeroed v1 total, and the v2 constants and
package cases are exported and documented.

## Changes in `.10`

Collection item validation now accepts marketplace listing entity IDs, including
32-character UUID-like IDs, while keeping post item IDs on the 13-character
Crockford validator. Listing IDs remain bounded to the path-safe entity-id
alphabet and length limits.

## Changes in `.9`

Open-world record parsing (`b8e0c255`), aligned with the social/v1
forward-compat rule:

- Marketplace **records** (shop, listing, review, review response, drop,
  watchlist, order receipt) tolerate unknown members on parse so shapes
  can grow additively.
- The **six** JWS attestation header/claim structs stay closed-world
  (`deny_unknown_fields`): purchase attestation header + claims, order
  receipt attestation header + claims, drop-edition attestation header +
  claims. An attestation is an opaque signed string, not an evolving
  record.

## Specification additions (no code change)

- **Encrypted Private Records (`pubky-priv-aead/v1`)** in `SPEC.md`: the sealed storage form of the watchlist, order receipts and badge checkpoints. It covers per-owner random data keys, HKDF-SHA256 record and path subkeys, HMAC-derived and listed entry paths under `/priv/pubky.app/marketplace/v2/s/`, the XChaCha20-Poly1305 envelope with associated data bound to the entry name (so the recovery file alone opens every entry after a directory listing), reader validation, writer obligations (read-back before deleting plaintext, never overwrite what does not open or does not validate, no plaintext fallback, no private paths in logs), the badge checkpoint schema and its v1 layout, the recovery key file and its handling, and a test vector. The record schemas are unchanged.

## Changes in `.8`

Marketplace drops (scheduled, limited-quantity releases) and numbered
drop editions on order receipts:

- **`PubkyAppMarketplaceDrop`** (`src/models/drop.rs`) — PUBLIC entity-id
  record at `/pub/pubky.app/marketplace/v1/drops/{drop_id}`, wired into
  `Resource::Drop`, the URI path parser, `PubkyAppObject`, and
  `dropUriBuilder` (exactly like listings) so Nexus can index it. Fields:
  base record shape, `dropId` (entity id), `title` 1–120, `description`
  0–2000, up to 10 unique seller-owned marketplace media URIs, closed-world
  `format` (only `"fcfs"` in this version — future formats are a version
  bump), `startsAt` + optional strictly-later `endsAt` (absent = ends by
  sell-out or cancellation; both are seller intent — the transaction
  service enforces the real schedule), 1–20 unique `listingIds` (the
  seller's OWN listings; the record owner is the listing owner by
  definition), `totalQuantity` 1–1,000,000, `perBuyerLimit` 1–100 and
  ≤ `totalQuantity`, and the declared `stockDisplay` policy
  (`exact`/`bands`/`hidden`; enforcement is server-side).
- **Drop edition attestation format**
  (`src/models/drop_edition_attestation.rs`) — the normative reference for
  the compact JWS (EdDSA/Ed25519) carried in a receipt's
  `editionAttestation`: closed-world header
  `{"alg":"EdDSA","typ":"pubky-drop-edition+v1"}` and `v: 1` claim set
  (`iss`, `buyer`, `seller`, `drop`, `edition`, `of` ≥ `edition`,
  `receipt`, `iat` derived from the receipt's payment instant — same
  deterministic-issuance doctrine as the receipt attestation), structural
  parsing, offline signature verification, receipt-record binding checks,
  and a `verify_edition_for_order_receipt` recipe that requires the
  record's `editionAttestation` AND `drop` object to both be present.
  Exported as `PubkyAppDropEditionAttestation` /
  `PubkyAppDropEditionAttestationClaims`.
- **Order receipt drop-edition fields** (`src/models/order_receipt.rs`) —
  two optional fields that must be present together or absent together:
  `editionAttestation` (same 32–4096 `[A-Za-z0-9._~-]` bounds as
  `receiptAttestation`) and `drop`
  (`PubkyAppOrderReceiptDrop { dropId, edition ≥ 1, of ≥ edition }`,
  closed-world). Absent fields are not serialized, so `.7` receipts
  round-trip byte-compatibly (covered by a compatibility test).
- Wiring: `Resource::Drop` + path parsing, `PubkyAppObject::Drop`,
  `dropUriBuilder`, wasm `createMarketplaceDrop` +
  `PubkyAppMarketplaceDrop.fromJson`/`toJson`,
  `parseDropEditionAttestation`, `verifyDropEditionAttestation`, exports,
  docs (`SPEC.md`), tests.

## Changes in `.7`

Portable private order receipts ("credible exit for orders") and the shop's
transaction-service declaration:

- **`PubkyAppMarketplaceOrderReceipt`** (`src/models/order_receipt.rs`) —
  private per-order record at
  `/priv/pubky.app/marketplace/v1/receipts/{receipt_id}` (the transaction
  service's receipt UUID, lowercase hyphenated). The buyer or seller writes
  their own copy of a completed order — role/owner binding enforced
  (`ownerPubky` must equal `buyerPubky` when `role == "buyer"`, `sellerPubky`
  when `"seller"`), distinct parties, money `total`, `paidAt`, and a
  32–4096-char `receiptAttestation` compact JWS — so a signed purchase
  history survives the marketplace operator's disappearance. Like the
  watchlist, deliberately NOT wired into `PubkyAppObject` / URI resource
  resolution; never indexed.
- **Order receipt attestation format**
  (`src/models/order_receipt_attestation.rs`) — the normative reference for
  the compact JWS (EdDSA/Ed25519) carried in `receiptAttestation`:
  closed-world header `{"alg":"EdDSA","typ":"pubky-order-receipt+v1"}` and
  `v: 1` claim set (`iss`, `buyer`, `seller`, raw `order` UUID — receipts
  are private, so no salted ref — `receipt`, `total_minor`, `currency`,
  `exponent`, `paid_at` UTC, `iat` derived from `paid_at`), structural
  parsing, offline signature verification (the `iss` pubky *is* the
  verification key), record-binding checks, and deterministic issuance (a
  given receipt always yields the same JWS). Exported as
  `PubkyAppOrderReceiptAttestation` / `PubkyAppOrderReceiptAttestationClaims`.
- **Shop `transactionService` field** (`src/models/shop.rs`) — optional
  HTTPS base URL of the marketplace transaction service the shop sells
  through (scheme exactly `https`, no credentials/query/fragment, ≤ 300
  chars). Clients MUST resolve transactional commands (checkout, offers,
  bids, orders) for the shop against this authority when present, falling
  back to their configured default when absent; absence keeps
  byte-compatibility with `.6` records.
- Wiring: `orderReceiptUriBuilder`, wasm `createMarketplaceOrderReceipt` +
  `PubkyAppMarketplaceOrderReceipt.fromJson`/`toJson`,
  `parseOrderReceiptAttestation`, `verifyOrderReceiptAttestation`, exports,
  docs (`SPEC.md`), tests.

## Changes in `.6`

Private cross-device watchlist record (the first `/priv/` record):

- **`PRIVATE_PATH` constant** (`/priv/`) — the homeserver's authenticated
  private storage prefix. Records under it are never watcher-indexed and are
  deliberately not wired into `PubkyAppObject` / URI resource resolution.
- **`PubkyAppWatchlist`** (`src/models/watchlist.rs`) — singleton private
  document at `/priv/pubky.app/marketplace/v1/watchlist.json`: LWW-mergeable
  `items` + `tombstones` keyed by `(listingOwnerPubky, listingId)` with
  integer-millisecond entry timestamps; caps of 500 each; keys unique across
  both lists (post-merge resolved state). Full rationale in `SPEC.md`.
- Wiring: `watchlistUriBuilder`, wasm `createWatchlist` +
  `PubkyAppWatchlist.fromJson`/`toJson`, exports, docs, tests.

## Changes in `.5`

- Listing path-ID validation accepts entity-id listing path ids (fixes
  rejection of valid non-timestamp listing ids).

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
