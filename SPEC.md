# Data model specification

_Version 0.6.0_

## Table of Contents

- [Data model specification](#data-model-specification)
  - [Table of Contents](#table-of-contents)
  - [Introduction](#introduction)
  - [Quick Start](#quick-start)
    - [Concepts:](#concepts)
  - [Data Models](#data-models)
    - [PubkyAppUser](#pubkyappuser)
    - [PubkyAppFile](#pubkyappfile)
    - [PubkyAppPost](#pubkyapppost)
    - [PubkyAppTag](#pubkyapptag)
    - [PubkyAppBookmark](#pubkyappbookmark)
    - [PubkyAppFollow](#pubkyappfollow)
    - [PubkyAppMute](#pubkyappmute)
    - [PubkyAppBlob](#pubkyappblob)
    - [PubkyAppLastRead](#pubkyapplastread)
    - [PubkyAppFeed](#pubkyappfeed)
      - [`feed` object (`PubkyAppFeedConfig`)](#feed-object-pubkyappfeedconfig)
    - [PubkyAppShop](#pubkyappshop)
    - [PubkyAppListing](#pubkyapplisting)
    - [PubkyAppMarketplaceDrop](#pubkyappmarketplacedrop)
    - [PubkyAppMarketplaceReview](#pubkyappmarketplacereview)
    - [Purchase Attestation (embedded JWS)](#purchase-attestation-embedded-jws)
    - [PubkyAppReviewResponse](#pubkyappreviewresponse)
    - [PubkyAppWatchlist (private)](#pubkyappwatchlist-private)
    - [PubkyAppMarketplaceOrderReceipt (private)](#pubkyappmarketplaceorderreceipt-private)
    - [Order Receipt Attestation (embedded JWS)](#order-receipt-attestation-embedded-jws)
    - [Drop Edition Attestation (embedded JWS)](#drop-edition-attestation-embedded-jws)
  - [Validation Rules](#validation-rules)
    - [Common Rules](#common-rules)
  - [License](#license)

---

## Introduction

This document specifies the data models and validation rules for the **Pubky.app** clients interactions. It defines the structure of data entities, their properties, and the validation rules to ensure data integrity and consistency. This is intended for developers building compatible libraries or clients.

This document is a faithful representation of our [Rust pubky.app models](https://github.com/pubky/pubky-app-specs/tree/main/src).

---

## Quick Start

Pubky.app models are designed for decentralized content sharing. The system uses a combination of timestamp-based IDs and Blake3-hashed IDs encoded in Crockford Base32 to ensure unique identifiers for each entity.

### Concepts:

- **Timestamp IDs** for sequential objects like posts and files.
- **Hash IDs** for content-based uniqueness (e.g., tags and bookmarks).
- **Validation Rules** ensure consistent and interoperable data formats.

---

## Data Models

### PubkyAppUser

**Description:** Represents a user's profile information.

**URI:** `/pub/pubky.app/profile.json`

| **Field** | **Type** | **Description**                         | **Validation Rules**                                                                         |
| --------- | -------- | --------------------------------------- | -------------------------------------------------------------------------------------------- |
| `name`    | String   | User's name.                            | Required. Length: 3–50 characters. Cannot be `"[DELETED]"`.                                  |
| `bio`     | String   | Short biography.                        | Optional. Maximum length: 160 characters.                                                    |
| `image`   | String   | URL to the user's profile image.        | Optional. Valid URL. Maximum length: 300 characters.                                         |
| `links`   | Array    | List of associated links (title + URL). | Optional. Maximum of 5 links, each with title (100 chars max) and valid URL (300 chars max). |
| `status`  | String   | User's current status.                  | Optional. Maximum length: 50 characters.                                                     |

**Validation Notes:**

- Reserved keyword `[DELETED]` cannot be used for `name`.
- Each `UserLink` in `links` must have a valid title and URL.

**Example: Valid User**

```json
{
  "name": "Alice",
  "bio": "Toxic maximalist.",
  "image": "pubky://user_id/pub/pubky.app/files/0000000000000",
  "links": [
    {
      "title": "GitHub",
      "url": "https://github.com/alice"
    }
  ],
  "status": "Exploring decentralized tech."
}
```

---

### PubkyAppFile

**Description:** Represents a file uploaded by the user, containing its metadata, including a reference to the actual blob of the file in `src` property.

**URI:** `/pub/pubky.app/files/:file_id`

| **Field**      | **Type** | **Description**             | **Validation Rules**                           |
| -------------- | -------- | --------------------------- | ---------------------------------------------- |
| `name`         | String   | Name of the file.           | Required. Must be 1-255 characters             |
| `created_at`   | Integer  | Unix timestamp of creation. | Required.                                      |
| `src`          | String   | File blob URL               | Required. must be a valid URL. Max length 1024 |
| `content_type` | String   | MIME type of the file.      | Required. Valid IANA mime types                |
| `size`         | Integer  | Size of the file in bytes.  | Required. Positive integer. Max size is 10Mb   |

**Validation Notes:**

- The `file_id` in the URI must be a valid **Timestamp ID**.

---

### PubkyAppPost

**Description:** Represents a user's post.

**URI:** `/pub/pubky.app/posts/:post_id`

| **Field**     | **Type** | **Description**                      | **Validation Rules**                                                       |
| ------------- | -------- | ------------------------------------ | -------------------------------------------------------------------------- |
| `content`     | String   | Content of the post.                 | Required. Max length: 2000 (short), 50000 (long). Cannot be `"[DELETED]"`. |
| `kind`        | String   | Type of post.                        | Required. Must be a valid `PubkyAppPostKind` value.                        |
| `parent`      | String   | URI of the parent post (if a reply). | Optional. Must be a valid URI if present.                                  |
| `embed`       | Object   | Reposted content (type + URI).       | Optional. URI must be valid if present.                                    |
| `attachments` | Array    | List of attachment URIs.             | Optional. Each must be a valid URI.                                        |
| `lock`        | String   | Lock server URL for protected posts. | Optional. If present, must be a valid `pubky://` URL with a host, up to 200 characters. Missing or `null` means unlocked. |

**Post Kinds:**

- `short`
- `long`
- `image`
- `video`
- `link`
- `file`
- `collection`

**Example: Valid Post**

```json
{
  "content": "Hello world! This is my first post.",
  "kind": "short",
  "parent": null,
  "embed": {
    "kind": "short",
    "uri": "pubky://user_id/pub/pubky.app/posts/0000000000000"
  },
  "attachments": ["pubky://user_id/pub/pubky.app/files/0000000000000"],
  "lock": "pubky://lock_server_id/pub/locks/0000000000000"
}
```

**Locking:**

Posts are unlocked by default. A post may include `lock` to advertise that the full post is protected behind a lock server. When present, `lock` must be a valid `pubky://` URL with a host, up to 200 characters. Consumers that receive JSON without `lock`, or JSON with `"lock": null`, must treat the post as a regular unlocked post.

**Note on `kind = collection`:**

Collection posts use a typed JSON envelope as their `content`. The envelope shape is:

```json
{
  "name": "AI papers",
  "description": "Best stuff",
  "cover_image": "pubky://userA/pub/pubky.app/files/0034A0X7NJ52C",
  "layout": "visual",
  "items": [
    "pubky://userA/pub/pubky.app/posts/0034A0X7NJ52A",
    "pubky://userB/pub/pubky.app/posts/0034A0X7NJ52B"
  ]
}
```

- `name`: required, 1 to 100 unicode scalars, non-whitespace-only.
- `description`: optional, max 500 scalars.
- `cover_image`: optional hero/cover image URL (max 200 chars). Validated as a general attachment URL — protocol must be `pubky`, `http`, or `https`.
- `layout`: optional, one of `grid`, `list`, `visual`; the creator's default layout for experiencing the collection. Absent = `grid`. Consumers must treat unrecognized values as `grid`.
- `items`: ordered list of pubky.app URIs (max 100). Each URI must be in exact canonical form — either a post URI `pubky://<pubky-id>/pub/pubky.app/posts/<post-id>` (94 chars) or a marketplace listing URI `pubky://<pubky-id>/pub/pubky.app/marketplace/v1/listings/<listing-id>`; any deviation (extra path segments, query, fragment, userinfo, etc.) is rejected.

For `kind = collection`, `parent`, `embed`, and `post.attachments` must be unset. The `content` field is bounded by 40000 scalars instead of the regular short/long caps.

---

### PubkyAppTag

**Description:** Represents a tag applied to a URI.

**URI:** `/pub/pubky.app/tags/:tag_id`

| **Field**    | **Type** | **Description**             | **Validation Rules**                                     |
| ------------ | -------- | --------------------------- | -------------------------------------------------------- |
| `uri`        | String   | URI of the tagged object.   | Required. Must be a valid URI.                           |
| `label`      | String   | Label for the tag.          | Required. Trimmed, lowercase. Max length: 20 characters. |
| `created_at` | Integer  | Unix timestamp of creation. | Required.                                                |

**Validation Notes:**

- The `tag_id` is a **Hash ID** derived from the `uri` and `label`.

---

### PubkyAppBookmark

**Description:** Represents a bookmark to a URI.

**URI:** `/pub/pubky.app/bookmarks/:bookmark_id`

| **Field**    | **Type** | **Description**        | **Validation Rules**           |
| ------------ | -------- | ---------------------- | ------------------------------ |
| `uri`        | String   | URI of the bookmark.   | Required. Must be a valid URI. |
| `created_at` | Integer  | Timestamp of creation. | Required.                      |

**Validation Notes:**

- The `bookmark_id` is a **Hash ID** derived from the `uri`.

---

### PubkyAppFollow

**Description:** Represents a follow relationship.

**URI:** `/pub/pubky.app/follows/:user_id`

| **Field**    | **Type** | **Description**        | **Validation Rules** |
| ------------ | -------- | ---------------------- | -------------------- |
| `created_at` | Integer  | Timestamp of creation. | Required.            |

---

### PubkyAppMute

**Description:** Represents a mute relationship (a user the author has muted).

**URI:** `/pub/pubky.app/mutes/:user_id`

| **Field**    | **Type** | **Description**        | **Validation Rules** |
| ------------ | -------- | ---------------------- | -------------------- |
| `created_at` | Integer  | Timestamp of creation. | Required.            |

**Validation Notes:**

- The `user_id` in the URI is the **Pubky ID** of the muted user (same pattern as follows).

---

### PubkyAppBlob

**Description:** Raw binary data backing an uploaded file. Stored as bytes on the homeserver, not as a JSON object.

**URI:** `/pub/pubky.app/blobs/:blob_id`

| **Field** | **Type** | **Description**              | **Validation Rules**                          |
| --------- | -------- | ---------------------------- | --------------------------------------------- |
| *(body)*  | Bytes    | Raw file content.            | Required. Non-empty. Max size 100 MB.         |

**Validation Notes:**

- The `blob_id` is a **Hash ID** derived from the Blake3 hash of the blob bytes.
- Unlike other models, the homeserver body is the raw byte payload itself (not JSON).

---

### PubkyAppLastRead

**Description:** Tracks the last-read notification timestamp for a user.

**URI:** `/pub/pubky.app/last_read`

| **Field**   | **Type** | **Description**                              | **Validation Rules**        |
| ----------- | -------- | -------------------------------------------- | --------------------------- |
| `timestamp` | Integer  | Last-read time (Unix epoch, **milliseconds**). | Required. Positive integer. |

**Validation Notes:**

- Single resource per user (no ID segment in the path).
- `timestamp` uses **milliseconds**, unlike `created_at` on other models which use microseconds.

---

### PubkyAppFeed

**Description:** Represents a feed configuration.

**URI:** `/pub/pubky.app/feeds/:feed_id`

| **Field**      | **Type**  | **Description**             | **Validation Rules**                    |
| -------------- | --------- | --------------------------- | --------------------------------------- |
| `feed`         | Object    | Feed filter/sort settings.  | Required. See `feed` object below.      |
| `name`         | String    | Display name of the feed.   | Required. Non-empty after trim.         |
| `created_at`   | Integer   | Unix timestamp of creation. | Required.                               |

#### `feed` object (`PubkyAppFeedConfig`)

| **Field**   | **Type** | **Description**                    | **Validation Rules**                                      |
| ----------- | -------- | ---------------------------------- | --------------------------------------------------------- |
| `tags`      | Array    | Tags for filtering.                | Optional. Max 5 tags. Each tag follows tag label rules.     |
| `domain_tags` | Array  | Domain tags for filtering.         | Optional. Max 5 tags. Each tag follows tag label rules.   |
| `reach`     | String   | Feed visibility scope.             | Required. One of: `following`, `followers`, `friends`, `all`, `wot`, `me`. |
| `layout`    | String   | Feed layout style.                 | Required. One of: `columns`, `wide`, `visual`, `list`.    |
| `sort`      | String   | Sort order.                        | Required. One of: `recent`, `popularity`.                   |
| `content`   | String   | Post kind to filter by.            | Optional. A valid `PubkyAppPostKind` value.               |

**Validation Notes:**

- The `feed_id` is a **Hash ID** derived from the serialized `feed` object.
- Tags and domain tags are trimmed, lowercased, and empty entries are removed on sanitize.

**Example: Valid Feed**

```json
{
  "feed": {
    "tags": ["crab", "rust"],
    "domain_tags": ["synonym"],
    "reach": "wot",
    "layout": "columns",
    "sort": "recent",
    "content": "video"
  },
  "name": "My Feed",
  "created_at": 1700000000
}
```

---

### PubkyAppShop

**Description:** Represents a seller's marketplace shop profile (singleton per user). All fields are serialized in camelCase and unknown fields are rejected.

**URI:** `/pub/pubky.app/marketplace/v1/shop.json`

| **Field**        | **Type** | **Description**                        | **Validation Rules**                                                              |
| ---------------- | -------- | -------------------------------------- | --------------------------------------------------------------------------------- |
| `schemaVersion`  | Integer  | Marketplace contract version.          | Required. Must be `1`.                                                            |
| `recordType`     | String   | Record discriminator.                  | Required. Must be `"shop"`.                                                       |
| `ownerPubky`     | String   | Pubky of the shop owner.               | Required. 52-character z-base-32 pubky.                                           |
| `revision`       | Integer  | Record revision.                       | Required. Positive safe integer.                                                  |
| `createdAt`      | String   | Creation datetime.                     | Required. ISO-8601 with offset (`Z` or `±HH:MM`).                                 |
| `updatedAt`      | String   | Last-update datetime.                  | Required. ISO-8601 with offset. Must not precede `createdAt`.                     |
| `name`           | String   | Shop display name.                     | Required. Trimmed. Length: 1–60 characters.                                       |
| `bio`            | String   | Shop description.                      | Required (may be empty). Trimmed. Maximum length: 1000 characters.                |
| `location`       | Object   | Public location.                       | Required. `countryCode` ISO 3166-1 alpha-2; optional `region` 1–100 characters.   |
| `avatarUrl`      | String   | Shop avatar media URI.                 | Optional. Must be a Pubky marketplace v1 URI.                                     |
| `bannerUrl`      | String   | Shop banner media URI.                 | Optional. Must be a Pubky marketplace v1 URI.                                     |
| `shippingPolicy` | String   | Default shipping policy.               | Required (may be empty). Trimmed. Maximum length: 4000 characters.                |
| `returnPolicy`   | String   | Default return policy.                 | Required (may be empty). Trimmed. Maximum length: 4000 characters.                |
| `vacationMode`   | Boolean  | Whether the shop is paused (vacation). | Required.                                                                          |
| `transactionService` | String | HTTPS base URL of the marketplace transaction service this shop sells through. | Optional. Must parse as a URL with scheme exactly `https`, no credentials, no query, no fragment; maximum length 300 characters. Absent fields are not serialized (records without it round-trip unchanged). |

**`transactionService` semantics:** when present, clients MUST resolve transactional commands (checkout, offers, bids, orders) for this shop against this authority, falling back to their configured default when absent. Two different services may register the same public listing — the shop record is the seller's declaration of which one is authoritative.

---

### PubkyAppListing

**Description:** Represents a marketplace listing published by a seller. All fields are serialized in camelCase and unknown fields are rejected. Money is always integer minor units (`amountMinor`, `currency`, `exponent`).

**URI:** `/pub/pubky.app/marketplace/v1/listings/:listing_id`

| **Field**            | **Type** | **Description**                          | **Validation Rules**                                                                                              |
| -------------------- | -------- | ---------------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| `schemaVersion`      | Integer  | Marketplace contract version.            | Required. Must be `1`.                                                                                             |
| `recordType`         | String   | Record discriminator.                    | Required. Must be `"listing"`.                                                                                     |
| `ownerPubky`         | String   | Pubky of the seller.                     | Required. 52-character z-base-32 pubky.                                                                            |
| `revision`           | Integer  | Record revision.                         | Required. Positive safe integer.                                                                                   |
| `createdAt`          | String   | Creation datetime.                       | Required. ISO-8601 with offset.                                                                                    |
| `updatedAt`          | String   | Last-update datetime.                    | Required. ISO-8601 with offset. Must not precede `createdAt`.                                                      |
| `listingId`          | String   | Listing identifier.                      | Required. Path-safe id that must equal the **Timestamp ID** in the record path.                                    |
| `state`              | String   | Listing lifecycle state.                 | Required. One of `active`, `paused`, `ended`, `removed`. Active listings need an enabled variant with stock.       |
| `title`              | String   | Listing title.                           | Required. Trimmed. Length: 3–80 characters.                                                                        |
| `description`        | String   | Listing description.                     | Required. Trimmed. Length: 1–10000 characters.                                                                     |
| `taxonomyVersion`    | Integer  | Category taxonomy version.               | Required. Integer between 1 and 1000000. The category tree and per-category attribute expectations are client configuration keyed by this number. |
| `categoryId`         | String   | Marketplace category.                    | Required. Kebab-case identifier, 1–120 characters.                                                                 |
| `condition`          | String   | Item condition.                          | Required. One of `new`, `like_new`, `excellent`, `good`, `fair`, `for_parts`.                                      |
| `attributes`         | Object   | Item specifics (key/value container).    | Optional. Map of at most 20 keys to a string or an array of 1–10 unique strings. Keys are lowercase alphanumeric identifiers with single `-`/`_` separators, 1–40 characters. Values are trimmed, 1–80 characters. Which keys a category expects is client configuration keyed by `taxonomyVersion`. |
| `conditionDetails`   | String   | Extra condition notes.                   | Optional. Trimmed. Maximum length: 1000 characters.                                                                |
| `tags`               | Array    | Search tags.                             | Required. Up to 10 unique, trimmed strings of 1–40 characters.                                                     |
| `location`           | Object   | Public location.                         | Required. Same rules as the shop location.                                                                         |
| `media`              | Array    | Media attachments.                       | Required. 1–13 entries; 1–12 images and at most 1 video; unique ids; URIs must be seller-owned marketplace media.  |
| `variants`           | Array    | Purchasable variants (SKUs).             | Required. 1–100 entries; unique ids and SKUs; quantity 0–1000000; media references must exist.                     |
| `sale`               | Object   | Sale terms.                              | Required. `format` is `fixed_price` (positive `unitPrice`, `acceptsOffers`) or `auction` (see below).              |
| `fulfillmentMethods` | Array    | Delivery methods.                        | Required. 1–3 unique values of `physical`, `digital`, `pickup`.                                                    |
| `package`            | Object   | Package facts (weight/dimensions).       | Required with `physical` fulfillment, forbidden otherwise.                                                         |
| `shippingOptions`    | Array    | Shipping options.                        | Up to 20 unique-id options (`free`, `flat`, `calculated`). Required non-empty with `physical`, forbidden otherwise. |
| `returnPolicy`       | Object   | Return policy.                           | Required. Return window (1–365 days) required iff returns are accepted.                                            |
| `digitalLock`        | Object   | Locks policy for digital delivery.       | Required with `digital` fulfillment, forbidden otherwise.                                                          |
| `adultOnly`          | Boolean  | Adult-content flag.                      | Required.                                                                                                          |

**Auction rules:** all auction prices must share one asset and exponent; `endsAt` must follow `startsAt`; reserve price must not be below the starting price; buy-now price must exceed the starting price; anti-sniping windows are 0–3600 seconds; auctions require exactly one variant. Variant price overrides and flat shipping prices must use the listing asset.

**Validation Notes:**

- The `listing_id` in the URI must be a valid **Timestamp ID** and must match the record's `listingId` field.

---

### PubkyAppMarketplaceDrop

**Description:** Represents a marketplace drop: a seller's scheduled, limited-quantity release bundling one or more of their own listings. This is a PUBLIC record (unlike the private order receipt) wired into the URI parser and `PubkyAppObject` so Nexus can index it. All fields are serialized in camelCase and unknown fields are rejected.

**URI:** `/pub/pubky.app/marketplace/v1/drops/:drop_id`

| **Field**        | **Type** | **Description**                              | **Validation Rules**                                                              |
| ---------------- | -------- | -------------------------------------------- | ---------------------------------------------------------------------------------- |
| `schemaVersion`  | Integer  | Marketplace contract version.                | Required. Must be `1`.                                                             |
| `recordType`     | String   | Record discriminator.                        | Required. Must be `"drop"`.                                                        |
| `ownerPubky`     | String   | Pubky of the seller.                         | Required. 52-character z-base-32 pubky.                                            |
| `revision`       | Integer  | Record revision.                             | Required. Positive safe integer.                                                   |
| `createdAt`      | String   | Creation datetime.                           | Required. ISO-8601 with offset (`Z` or `±HH:MM`).                                  |
| `updatedAt`      | String   | Last-update datetime.                        | Required. ISO-8601 with offset. Must not precede `createdAt`.                      |
| `dropId`         | String   | Drop identifier.                             | Required. Path-safe entity ID (1–128 chars of `[A-Za-z0-9_-]`); must equal the id in the record path. |
| `title`          | String   | Drop title.                                  | Required. Trimmed. Length: 1–120 characters.                                       |
| `description`    | String   | Drop description.                            | Required (may be empty). Trimmed. Maximum length: 2000 characters.                 |
| `media`          | Array    | Promotional media URIs.                      | Required (may be empty). At most 10 unique entries; each must be a Pubky marketplace v1 URI owned by the drop's seller (same rule as listing media URLs). |
| `format`         | String   | How the drop sells.                          | Required. Only `"fcfs"` (first-come, first-served) in this version — the enum is closed-world, so any future format is a schema version bump. |
| `startsAt`       | String   | The seller's declared start.                 | Required. ISO-8601 with offset.                                                    |
| `endsAt`         | String   | The seller's declared end.                   | Optional. ISO-8601 with offset; when present must be strictly after `startsAt`. Absent means the drop ends only by sell-out or seller cancellation. Absent fields are not serialized. |
| `listingIds`     | Array    | The seller's OWN listings bundled into the drop. | Required. 1–20 unique path-safe entity IDs. The record owner is the listing owner by definition — no cross-owner reference form exists. |
| `totalQuantity`  | Integer  | Total units across the drop.                 | Required. Integer between 1 and 1000000.                                           |
| `perBuyerLimit`  | Integer  | Per-buyer purchase cap.                      | Required. Integer between 1 and 100; must not exceed `totalQuantity`.              |
| `stockDisplay`   | String   | Declared stock-visibility policy.            | Required. One of `"exact"`, `"bands"`, `"hidden"` — how much remaining-stock detail the seller wants the public projection to reveal. |

**Authority note:** `startsAt`/`endsAt` are the seller's declared schedule intent — the marketplace transaction service enforces the real sale window and the real stock counters. Likewise `stockDisplay` is the seller's declared policy; ENFORCEMENT is server-side (the service, which holds the counters, decides what each stock query answers).

---

### PubkyAppMarketplaceReview

**Description:** Represents a marketplace review of a trade counterparty. All fields are serialized in camelCase and unknown fields are rejected.

**URI:** `/pub/pubky.app/marketplace/v1/reviews/:review_id`

| **Field**                | **Type** | **Description**                        | **Validation Rules**                                                             |
| ------------------------ | -------- | -------------------------------------- | --------------------------------------------------------------------------------- |
| `schemaVersion`          | Integer  | Marketplace contract version.          | Required. Must be `1`.                                                            |
| `recordType`             | String   | Record discriminator.                  | Required. Must be `"review"`.                                                     |
| `ownerPubky`             | String   | Pubky of the reviewer.                 | Required. 52-character z-base-32 pubky.                                           |
| `revision`               | Integer  | Record revision.                       | Required. Positive safe integer.                                                  |
| `createdAt`              | String   | Creation datetime.                     | Required. ISO-8601 with offset.                                                   |
| `updatedAt`              | String   | Last-update datetime.                  | Required. ISO-8601 with offset. Must not precede `createdAt`.                     |
| `reviewId`               | String   | Review identifier.                     | Required. Must equal the **Hash ID** in the record path.                          |
| `subjectPubky`           | String   | Pubky of the reviewed user.            | Required. 52-character z-base-32 pubky.                                           |
| `listingOwnerPubky`      | String   | Pubky of the reviewed listing's owner. | Required. 52-character z-base-32 pubky.                                           |
| `listingId`              | String   | Reviewed listing identifier.           | Required. Path-safe identifier, 1–128 characters.                                 |
| `role`                   | String   | Review direction.                      | Required. `buyer_reviewing_seller` or `seller_reviewing_buyer`.                   |
| `ratings`                | Object   | Star ratings.                          | Required. `overall` 1–5; optional `itemAccuracy`, `shipping`, `communication` 1–5. |
| `text`                   | String   | Review text.                           | Required. Trimmed. Length: 1–5000 characters.                                     |
| `eligibilityAttestation` | String   | Proof of review eligibility.           | Required. 32–4096 characters of `[A-Za-z0-9._~-]`.                                |

**Validation Notes:**

- The `review_id` is a **Hash ID** derived from `"{listing_uri}:{subject_pubky}:{role}"`, where `listing_uri` is `pubky://<listingOwnerPubky>/pub/pubky.app/marketplace/v1/listings/<listingId>`.

---

### Purchase Attestation (embedded JWS)

**Description:** The normative format of the value carried in a review record's `eligibilityAttestation` when the review is attested by a marketplace transaction authority. It is a compact JWS (RFC 7515) signed with EdDSA/Ed25519 (RFC 8037): `base64url(header).base64url(claims).base64url(signature)`, unpadded. The attestation attests the **purchase**, not the review text: record revisions leave it unchanged, and it carries no expiry.

**Header (closed-world, exactly these fields):**

```json
{ "alg": "EdDSA", "typ": "pubky-purchase-attestation+v1" }
```

**Claims (version `v: 1`, closed-world — unknown claims are rejected):**

| **Claim**      | **Type** | **Description**                             | **Validation Rules**                                                                  |
| -------------- | -------- | ------------------------------------------- | ------------------------------------------------------------------------------------- |
| `v`            | Integer  | Claim-set version.                          | Required. Must be `1`.                                                                 |
| `iss`          | String   | Attestor pubky.                             | Required. 52-character z-base-32 pubky; decodes to the Ed25519 verification key.       |
| `sub`          | String   | Reviewer pubky.                             | Required. Must equal the review record's `ownerPubky`.                                 |
| `cpk`          | String   | Counterparty pubky.                         | Required. Must equal the review record's `subjectPubky`.                               |
| `role`         | String   | Review direction.                           | Required. `buyer_reviewing_seller` or `seller_reviewing_buyer`; must match the record. |
| `listing`      | String   | Canonical listing URI.                      | Required. Must match the record's `listingOwnerPubky` + `listingId`.                   |
| `order_ref`    | String   | Attestor-salted Blake3 of the order UUID.   | Required. 64 lowercase hex characters. Opaque; only the attestor can link it back.     |
| `completed_on` | String   | Order completion date.                      | Required. `YYYY-MM-DD` (day granularity, deliberately no finer).                       |
| `amount_band`  | String   | Log-decade amount band.                     | Optional. `{CURRENCY}:{magnitude}`, e.g. `SAT:5`; magnitude 0–18. Emitted only under both-sides consent (seller standing preference AND per-review buyer opt-in). |
| `iat`          | Integer  | Issuance time (UNIX seconds).               | Required. Positive safe integer.                                                       |

**Verification recipe (offline, no issuer round-trip):**

1. Parse the compact JWS; reject unknown header fields, unknown claims, and unknown versions.
2. Decode `iss` from z-base-32 — that *is* the Ed25519 verification key. Verify the signature over `base64url(header) || '.' || base64url(claims)`.
3. Check bindings against the review record: `sub == ownerPubky`, `cpk == subjectPubky`, `listing` matches, `role` matches.
4. Accept as **verified** only if `iss` is on your own attestor trust list. The signature proves key possession, never legitimacy.

Exact amounts, timestamps finer than a day, addresses, payment identifiers, and bearer material are prohibited in claims.

---

### PubkyAppReviewResponse

**Description:** Represents the review subject's single revisable response to a marketplace review, published on the **responder's** homeserver. All fields are serialized in camelCase and unknown fields are rejected.

**URI:** `/pub/pubky.app/marketplace/v1/review_responses/:review_id`

| **Field**       | **Type** | **Description**                          | **Validation Rules**                                                              |
| --------------- | -------- | ---------------------------------------- | ---------------------------------------------------------------------------------- |
| `schemaVersion` | Integer  | Marketplace contract version.            | Required. Must be `1`.                                                             |
| `recordType`    | String   | Record discriminator.                    | Required. Must be `"review_response"`.                                             |
| `ownerPubky`    | String   | Pubky of the responder.                  | Required. 52-character z-base-32 pubky.                                            |
| `revision`      | Integer  | Record revision.                         | Required. Positive safe integer.                                                   |
| `createdAt`     | String   | Creation datetime.                       | Required. ISO-8601 with offset.                                                    |
| `updatedAt`     | String   | Last-update datetime.                    | Required. ISO-8601 with offset. Must not precede `createdAt`.                      |
| `reviewId`      | String   | Subject review's identifier.             | Required. Must equal the ID in the record path.                                    |
| `reviewUri`     | String   | Canonical URI of the subject review.     | Required. Must reference the same `reviewId`; must not be on the responder's own homeserver. |
| `text`          | String   | Response text.                           | Required. Trimmed. Length: 1–5000 characters.                                      |

**Validation Notes:**

- The path ID **equals the subject review's ID** — one response per review, revisable via `revision`.
- **Authorization is structural, not cryptographic:** indexers accept a response only when the response record's `ownerPubky` equals the subject review's `subjectPubky`. An impostor's response fails that check without any signature machinery. No attestation is carried.

---

### PubkyAppWatchlist (private)

**Description:** The user's private marketplace watchlist — the first record under `/priv/`, the homeserver's authenticated private storage. A watchlist reveals purchase intent, so unlike every `/pub/pubky.app/` record it must not be world-readable, directory-listable, or indexable; the homeserver refuses reads, listings, and writes on `/priv/` paths from anyone but the owner's own sessions. This record is deliberately **not** wired into `PubkyAppObject` or the URI parser's resource resolution: watchers and indexers never see it. Sessions need the `/priv/pubky.app/:rw` capability to touch it.

**URI:** `/priv/pubky.app/marketplace/v1/watchlist.json`

It is a **single revisioned document** (singleton per user, like `shop.json`) rather than one record per watched listing because: (a) watch/unwatch toggles are high-churn, and a single document makes each sync one `PUT` instead of a create/delete stream; (b) merge needs items and tombstones resolved together atomically — two files could tear; and (c) private storage has no index that would benefit from per-item paths.

| **Field**       | **Type** | **Description**                            | **Validation Rules**                                                              |
| --------------- | -------- | ------------------------------------------ | ---------------------------------------------------------------------------------- |
| `schemaVersion` | Integer  | Marketplace contract version.              | Required. Must be `1`.                                                             |
| `recordType`    | String   | Record discriminator.                      | Required. Must be `"watchlist"`.                                                   |
| `ownerPubky`    | String   | Pubky of the watchlist owner.              | Required. 52-character z-base-32 pubky.                                            |
| `revision`      | Integer  | Document revision.                         | Required. Positive safe integer.                                                   |
| `createdAt`     | String   | Creation datetime.                         | Required. ISO-8601 with offset.                                                    |
| `updatedAt`     | String   | Last-update datetime.                      | Required. ISO-8601 with offset. Must not precede `createdAt`.                      |
| `items`         | Array    | Actively watched listings.                 | Required (may be empty). At most 500 entries.                                      |
| `tombstones`    | Array    | Removed watches retained for merge.        | Required (may be empty). At most 500 entries; clients prune oldest-first.          |

`items[]` entries:

| **Field**           | **Type** | **Validation Rules**                                              |
| ------------------- | -------- | ----------------------------------------------------------------- |
| `listingOwnerPubky` | String   | Required. 52-character z-base-32 pubky of the seller.             |
| `listingId`         | String   | Required. Path-safe entity ID (1–128 chars of `[A-Za-z0-9_-]`).   |
| `watchedAtMs`       | Integer  | Required. Positive safe integer of epoch **milliseconds**.        |

`tombstones[]` entries carry the same `listingOwnerPubky` / `listingId` plus `removedAtMs` under the same rules.

**Validation Notes:**

- **Key uniqueness:** every `(listingOwnerPubky, listingId)` key appears at most once across `items` **and** `tombstones` combined. The document is the post-merge resolved state: a listing is either watched or removed, never both.
- **Entry timestamps are integer milliseconds** (not ISO-8601 like the document-level datetimes) on purpose: they are last-write-wins merge keys that clients compare numerically, immune to offset-formatting differences between writers.
- **Merge rule (normative for clients):** per listing key, the entry with the greater timestamp wins (`watchedAtMs` vs `removedAtMs`); ties resolve to the tombstone (deletion wins). The merged document is written back with `revision` incremented.

---

### PubkyAppMarketplaceOrderReceipt (private)

**Description:** A PRIVATE portable order receipt — the buyer's or seller's own durable copy of a completed order, written to their OWN homeserver. The marketplace transaction service holds the canonical order state, but a service is an operator that can disappear; this record is the **credible exit for orders**: each trade party keeps a signed, self-contained receipt (the embedded `receiptAttestation` JWS is offline-verifiable) on storage they control, so a purchase history survives the operator. All fields are serialized in camelCase and unknown fields are rejected.

**URI:** `/priv/pubky.app/marketplace/v1/receipts/:receipt_id`

**Privacy rationale:** like the watchlist, this is a `/priv/` record — an order history reveals counterparties, amounts, and purchase timing, so it must never be world-readable, directory-listable, or indexable; the homeserver refuses reads, listings, and writes on `/priv/` paths from anyone but the owner's own sessions. It is deliberately **not** wired into `PubkyAppObject` or the URI parser's resource resolution: watchers and indexers never see it. Unlike the watchlist singleton, receipts are one record per order under `receipts/:receipt_id` (the transaction service's receipt UUID, lowercase hyphenated): receipts are immutable facts, not merge targets, and per-id paths let a client sync incrementally instead of rewriting one growing document.

| **Field**            | **Type** | **Description**                              | **Validation Rules**                                                              |
| -------------------- | -------- | -------------------------------------------- | ---------------------------------------------------------------------------------- |
| `schemaVersion`      | Integer  | Marketplace contract version.                | Required. Must be `1`.                                                             |
| `recordType`         | String   | Record discriminator.                        | Required. Must be `"order_receipt"`.                                               |
| `ownerPubky`         | String   | Pubky of the record owner (a trade party).   | Required. 52-character z-base-32 pubky. Must equal `buyerPubky` when `role` is `"buyer"` and `sellerPubky` when `role` is `"seller"`. |
| `revision`           | Integer  | Record revision.                             | Required. Positive safe integer.                                                   |
| `createdAt`          | String   | Creation datetime.                           | Required. ISO-8601 with offset (`Z` or `±HH:MM`).                                  |
| `updatedAt`          | String   | Last-update datetime.                        | Required. ISO-8601 with offset. Must not precede `createdAt`.                      |
| `role`               | String   | The record owner's side of the order.        | Required. `"buyer"` or `"seller"`.                                                 |
| `receiptId`          | String   | The service's receipt UUID.                  | Required. Lowercase hyphenated UUID (8-4-4-4-12). Must equal the id in the record path. |
| `orderId`            | String   | The order UUID the receipt settles.          | Required. Lowercase hyphenated UUID (8-4-4-4-12).                                  |
| `buyerPubky`         | String   | Pubky of the buyer.                          | Required. 52-character z-base-32 pubky. Must differ from `sellerPubky`.            |
| `sellerPubky`        | String   | Pubky of the seller.                         | Required. 52-character z-base-32 pubky.                                            |
| `total`              | Object   | Order total in integer minor units.          | Required. Money object (`amountMinor`, `currency`, `exponent`); positive amount, uppercase 3–12 character asset code, exponent 0–18. |
| `paidAt`             | String   | Payment confirmation / receipt creation.     | Required. ISO-8601 with offset.                                                    |
| `receiptAttestation` | String   | Compact JWS attesting the receipt.           | Required. 32–4096 characters of `[A-Za-z0-9._~-]` (same bounds and charset as the review record's `eligibilityAttestation`). |
| `editionAttestation` | String   | Compact JWS attesting the drop edition (see [Drop Edition Attestation](#drop-edition-attestation-embedded-jws)). | Optional. Same bounds and charset as `receiptAttestation`. Must be present exactly when `drop` is present. Absent fields are not serialized (pre-drop receipts round-trip unchanged). |
| `drop`               | Object   | Drop display object: `{ dropId, edition, of }`. | Optional. `dropId` path-safe entity ID; `edition` positive integer; `of` positive integer, never below `edition`. Must be present exactly when `editionAttestation` is present. Absent fields are not serialized. |

---

### Order Receipt Attestation (embedded JWS)

**Description:** The normative format of the value carried in an order receipt record's `receiptAttestation`. It is a compact JWS (RFC 7515) signed with EdDSA/Ed25519 (RFC 8037): `base64url(header).base64url(claims).base64url(signature)`, unpadded.

**Header (closed-world, exactly these fields):**

```json
{ "alg": "EdDSA", "typ": "pubky-order-receipt+v1" }
```

**Claims (version `v: 1`, closed-world — unknown claims are rejected; issuers serialize claims in exactly this order):**

| **Claim**     | **Type** | **Description**                             | **Validation Rules**                                                                  |
| ------------- | -------- | ------------------------------------------- | -------------------------------------------------------------------------------------- |
| `v`           | Integer  | Claim-set version.                          | Required. Must be `1`.                                                                 |
| `iss`         | String   | Attestor pubky.                             | Required. 52-character z-base-32 pubky; decodes to the Ed25519 verification key.       |
| `buyer`       | String   | Buyer pubky.                                | Required. Must equal the receipt record's `buyerPubky`.                                |
| `seller`      | String   | Seller pubky.                               | Required. Must equal the receipt record's `sellerPubky`.                               |
| `order`       | String   | Raw order UUID.                             | Required. Lowercase hyphenated UUID; must equal the record's `orderId`.                |
| `receipt`     | String   | Receipt UUID.                               | Required. Lowercase hyphenated UUID; must equal the record's `receiptId`.              |
| `total_minor` | Integer  | Order total in integer minor units.         | Required. Positive safe integer; must equal the record total's `amountMinor`.          |
| `currency`    | String   | Asset code.                                 | Required. Same rules as money `currency`; must equal the record total's `currency`.    |
| `exponent`    | Integer  | Minor/major decimal places.                 | Required. 0–18; must equal the record total's `exponent`.                              |
| `paid_at`     | String   | Exact instant of payment confirmation.      | Required. ISO-8601 **UTC** (`Z` offset); must equal the record's `paidAt`.             |
| `iat`         | Integer  | Issuance time (UNIX seconds).               | Required. Must equal the epoch seconds of the `paid_at` instant.                       |

**Why the raw order id (unlike the public purchase attestation):** the purchase attestation travels inside a world-readable review, so its `order_ref` is an attestor-salted hash that nobody but the attestor can link back to an order. Receipts are private documents under `/priv/` that only the trade parties hold — there is no third-party observer to protect the linkage from, and the raw `order` UUID is exactly what makes the receipt actionable against the service (disputes, exports, audits) after the operator disappears.

**Determinism:** issuance is deterministic per receipt. The claims serialize in a fixed field order, `paid_at` is the canonical UTC serialization of the payment instant, `iat` is derived from that same instant (never the signing wall clock), and Ed25519 signatures are deterministic — so a given receipt always yields the same compact JWS, byte for byte.

**Verification recipe (offline, no issuer round-trip):**

1. Parse the compact JWS; reject unknown header fields, unknown claims, and unknown versions.
2. Decode `iss` from z-base-32 — that *is* the Ed25519 verification key. Verify the signature over `base64url(header) || '.' || base64url(claims)`.
3. Check bindings against the receipt record: `buyer == buyerPubky`, `seller == sellerPubky`, `order == orderId`, `receipt == receiptId`, `total_minor`/`currency`/`exponent` equal the record's `total`, and `paid_at == paidAt`.
4. Accept as **verified** only if `iss` is on your own attestor trust list. The signature proves key possession, never legitimacy.

---

### Drop Edition Attestation (embedded JWS)

**Description:** The normative format of the value carried in an order receipt record's `editionAttestation`. It attests that one order bought edition `edition` out of `of` total units of the seller's drop. It is a compact JWS (RFC 7515) signed with EdDSA/Ed25519 (RFC 8037): `base64url(header).base64url(claims).base64url(signature)`, unpadded.

**Header (closed-world, exactly these fields):**

```json
{ "alg": "EdDSA", "typ": "pubky-drop-edition+v1" }
```

**Claims (version `v: 1`, closed-world — unknown claims are rejected; issuers serialize claims in exactly this order):**

| **Claim**  | **Type** | **Description**                              | **Validation Rules**                                                                  |
| ---------- | -------- | -------------------------------------------- | -------------------------------------------------------------------------------------- |
| `v`        | Integer  | Claim-set version.                           | Required. Must be `1`.                                                                 |
| `iss`      | String   | Attestor pubky.                              | Required. 52-character z-base-32 pubky; decodes to the Ed25519 verification key.       |
| `buyer`    | String   | Buyer pubky.                                 | Required. Must equal the receipt record's `buyerPubky`.                                |
| `seller`   | String   | Seller pubky (the drop owner).               | Required. Must equal the receipt record's `sellerPubky`.                               |
| `drop`     | String   | The drop's entity id.                        | Required. Path-safe entity ID; must equal the record's `drop.dropId`.                  |
| `edition`  | Integer  | This order's edition number, 1-based.        | Required. Positive safe integer; must equal the record's `drop.edition`.               |
| `of`       | Integer  | The drop's `totalQuantity` at issuance.      | Required. Positive safe integer, never below `edition`; must equal the record's `drop.of`. |
| `receipt`  | String   | Receipt UUID.                                | Required. Lowercase hyphenated UUID; must equal the record's `receiptId`.              |
| `iat`      | Integer  | Issuance time (UNIX seconds).                | Required. Positive safe integer. Deterministic per receipt — derived from the receipt's payment instant, never the signing wall clock (same doctrine as the receipt attestation). |

**Determinism:** issuance is deterministic per receipt. The claims serialize in a fixed field order, `iat` is derived from the receipt (never the signing wall clock), and Ed25519 signatures are deterministic — so a given receipt always yields the same compact JWS, byte for byte.

**Verification recipe (offline, no issuer round-trip):**

1. Parse the compact JWS; reject unknown header fields, unknown claims, and unknown versions.
2. Decode `iss` from z-base-32 — that *is* the Ed25519 verification key. Verify the signature over `base64url(header) || '.' || base64url(claims)`.
3. Check bindings against the receipt record (which must carry BOTH `editionAttestation` and the `drop` object): `receipt == receiptId`, `buyer == buyerPubky`, `seller == sellerPubky`, and `drop`/`edition`/`of` equal the record's `drop` object fields.
4. Accept as **verified** only if `iss` is on your own attestor trust list. The signature proves key possession, never legitimacy.

---

## Validation Rules

### Common Rules

1. **Timestamp IDs:** 13-character Crockford Base32 strings derived from timestamps (in microseconds).
2. **Hash IDs:** First half of the bytes from the resulting Blake3-hashed strings encoded in Crockford Base32.
3. **URLs:** All URLs must pass standard validation.

---

## License

This specification is released under the MIT License.
