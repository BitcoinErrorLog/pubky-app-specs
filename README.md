# Pubky.app Data Model Specification

_Version 0.4.0_

> ⚠️ **Warning: Rapid Development Phase**  
> This specification is in an **early development phase** and is evolving quickly. Expect frequent changes and updates as the system matures. Consider this a **v0 draft**.
>
> When we reach the first stable, long-term support version of the schemas, paths will adopt the format: `pubky.app/v1/` to indicate compatibility and stability.

### JS package

The package is available as an npm module [pubky-app-specs](https://www.npmjs.com/package/pubky-app-specs). Alternatively, you can build from source using the provided build scripts:

```bash
cd pkg
npm run build
```

Test with:

```bash
cd pkg
npm run install
npm run test
```

Examples with:

```bash
cd pkg
npm run example
```

---

## Table of Contents

- [Pubky.app Data Model Specification](#pubkyapp-data-model-specification)
    - [JS package](#js-package)
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
    - [PubkyAppFeed](#pubkyappfeed)
    - [PubkyAppShop](#pubkyappshop)
    - [PubkyAppListing](#pubkyapplisting)
    - [PubkyAppMarketplaceReview](#pubkyappmarketplacereview)
  - [Validation Rules](#validation-rules)
    - [Common Rules](#common-rules)
  - [License](#license)

---

## Introduction

This document specifies the data models and validation rules for the **Pubky.app** clients interactions. It defines the structure of data entities, their properties, and the validation rules to ensure data integrity and consistency. This is intended for developers building compatible libraries or clients.

This document intents to be a faithful representation of our [Rust pubky.app models](https://github.com/pubky/pubky-app-specs/tree/main/src). If you intend to develop in Rust, use them directly. In case of disagreement between this document and the Rust implementation, the Rust implementation prevails.

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

**Post Kinds:**

- `short`
- `long`
- `image`
- `video`
- `link`
- `file`

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
  "attachments": ["pubky://user_id/pub/pubky.app/files/0000000000000"]
}
```

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

### PubkyAppFeed

**Description:** Represents a feed configuration.

**URI:** `/pub/pubky.app/feeds/:feed_id`

| **Field** | **Type** | **Description**                           | **Validation Rules**               |
| --------- | -------- | ----------------------------------------- | ---------------------------------- |
| `tags`    | Array    | List of tags for filtering.               | Optional. Strings must be trimmed. |
| `reach`   | String   | Feed visibility (e.g., `all`, `friends`). | Required. Must be a valid reach.   |
| `layout`  | String   | Feed layout style (e.g., `columns`).      | Required. Must be valid layout.    |
| `sort`    | String   | Sort order (e.g., `recent`).              | Required. Must be valid sort.      |
| `content` | String   | Type of content filtered.                 | Optional.                          |
| `name`    | String   | Name of the feed.                         | Required.                          |

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
| `taxonomyVersion`    | Integer  | Category taxonomy version.               | Required. Must be `1`.                                                                                             |
| `categoryId`         | String   | Marketplace category.                    | Required. Kebab-case identifier, 1–120 characters.                                                                 |
| `condition`          | String   | Item condition.                          | Required. One of `new`, `like_new`, `excellent`, `good`, `fair`, `for_parts`.                                      |
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

## Validation Rules

### Common Rules

1. **Timestamp IDs:** 13-character Crockford Base32 strings derived from timestamps (in microseconds).
2. **Hash IDs:** First half of the bytes from the resulting Blake3-hashed strings encoded in Crockford Base32.
3. **URLs:** All URLs must pass standard validation.

---

## License

This specification is released under the MIT License.
