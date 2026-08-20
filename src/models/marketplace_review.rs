use crate::{
    models::marketplace::{validate_base_record, validate_entity_id, validate_pubky},
    traits::{HasIdPath, HashId, Validatable},
    APP_PATH, PUBLIC_PATH,
};
use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
use crate::traits::Json;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

// Validation (mirroring the pubky-app commerce review record schema)
const REVIEW_RECORD_TYPE: &str = "review";
const MAX_REVIEW_TEXT_LENGTH: usize = 5_000;
const MIN_RATING: i64 = 1;
const MAX_RATING: i64 = 5;
const MIN_ATTESTATION_LENGTH: usize = 32;
const MAX_ATTESTATION_LENGTH: usize = 4_096;

/// Which side of the trade the review covers.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum PubkyAppReviewRole {
    BuyerReviewingSeller,
    SellerReviewingBuyer,
}

impl PubkyAppReviewRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            PubkyAppReviewRole::BuyerReviewingSeller => "buyer_reviewing_seller",
            PubkyAppReviewRole::SellerReviewingBuyer => "seller_reviewing_buyer",
        }
    }
}

/// Star ratings on a 1-5 integer scale. Only `overall` is required.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PubkyAppReviewRatings {
    pub overall: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_accuracy: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shipping: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub communication: Option<i64>,
}

impl PubkyAppReviewRatings {
    fn validate(&self) -> Result<(), String> {
        let all_ratings = [
            Some(self.overall),
            self.item_accuracy,
            self.shipping,
            self.communication,
        ];
        for rating in all_ratings.into_iter().flatten() {
            if !(MIN_RATING..=MAX_RATING).contains(&rating) {
                return Err(format!(
                    "Validation Error: ratings must be integers between {MIN_RATING} and {MAX_RATING}"
                ));
            }
        }
        Ok(())
    }
}

/// Represents a marketplace review of a trade counterparty.
///
/// URI: /pub/pubky.app/marketplace/v1/reviews/:review_id
///
/// Example URI:
///
/// `/pub/pubky.app/marketplace/v1/reviews/FPB0AM9S93Q3M1GFY1KV09GMQM`
///
/// Where review_id is
/// Crockford-base32(Blake3("{reviewed_listing_uri}:{subject_pubky}:{role}")[:half])
/// and must match the record's `reviewId` field.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PubkyAppMarketplaceReview {
    /// Marketplace contract version, always `1`.
    pub schema_version: i64,
    /// Record discriminator, always `"review"`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub record_type: String,
    /// z-base-32 pubky of the reviewer.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub owner_pubky: String,
    /// Monotonically increasing record revision, starting at 1.
    pub revision: i64,
    /// ISO-8601 creation datetime with offset.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub created_at: String,
    /// ISO-8601 last-update datetime with offset. Must not precede `created_at`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub updated_at: String,
    /// Must match the hash ID in the record's path.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub review_id: String,
    /// z-base-32 pubky of the user being reviewed.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub subject_pubky: String,
    /// z-base-32 pubky of the seller who owns the reviewed listing.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub listing_owner_pubky: String,
    /// Identifier of the reviewed listing.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub listing_id: String,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub role: PubkyAppReviewRole,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub ratings: PubkyAppReviewRatings,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub text: String,
    /// Opaque proof that the reviewer is eligible to review this trade.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub eligibility_attestation: String,
}

impl PubkyAppMarketplaceReview {
    /// Creates a new `PubkyAppMarketplaceReview` instance and sanitizes it.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        owner_pubky: String,
        revision: i64,
        created_at: String,
        updated_at: String,
        review_id: String,
        subject_pubky: String,
        listing_owner_pubky: String,
        listing_id: String,
        role: PubkyAppReviewRole,
        ratings: PubkyAppReviewRatings,
        text: String,
        eligibility_attestation: String,
    ) -> Self {
        Self {
            schema_version: crate::models::marketplace::MARKETPLACE_SCHEMA_VERSION,
            record_type: REVIEW_RECORD_TYPE.to_string(),
            owner_pubky,
            revision,
            created_at,
            updated_at,
            review_id,
            subject_pubky,
            listing_owner_pubky,
            listing_id,
            role,
            ratings,
            text,
            eligibility_attestation,
        }
        .sanitize()
    }
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl PubkyAppMarketplaceReview {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = fromJson))]
    pub fn from_json(js_value: &JsValue) -> Result<Self, String> {
        Self::import_json(js_value)
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = toJson))]
    pub fn to_json(&self) -> Result<JsValue, String> {
        self.export_json()
    }

    /// Getter for `text`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn text(&self) -> String {
        self.text.clone()
    }

    /// Getter for `listing_id`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn listing_id(&self) -> String {
        self.listing_id.clone()
    }

    /// Getter for `subject_pubky`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn subject_pubky(&self) -> String {
        self.subject_pubky.clone()
    }
}

#[cfg(target_arch = "wasm32")]
impl Json for PubkyAppMarketplaceReview {}

impl HasIdPath for PubkyAppMarketplaceReview {
    const PATH_SEGMENT: &'static str = "marketplace/v1/reviews/";

    fn create_path(id: &str) -> String {
        [PUBLIC_PATH, APP_PATH, Self::PATH_SEGMENT, id].concat()
    }
}

impl HashId for PubkyAppMarketplaceReview {
    /// Review ID is created based on the hash of the reviewed listing URI,
    /// the reviewed subject, and the review role.
    fn get_id_data(&self) -> String {
        let listing_uri =
            crate::listing_uri_builder(self.listing_owner_pubky.clone(), self.listing_id.clone());
        format!(
            "{}:{}:{}",
            listing_uri,
            self.subject_pubky,
            self.role.as_str()
        )
    }
}

impl Validatable for PubkyAppMarketplaceReview {
    fn sanitize(self) -> Self {
        PubkyAppMarketplaceReview {
            text: self.text.trim().to_string(),
            ..self
        }
    }

    fn validate(&self, id: Option<&str>) -> Result<(), String> {
        // Validate the review ID (hash regeneration) and its match with the record
        if let Some(id) = id {
            self.validate_id(id)?;
            if self.review_id != id {
                return Err("Validation Error: reviewId does not match the review path ID".into());
            }
        }

        validate_base_record(
            self.schema_version,
            &self.record_type,
            REVIEW_RECORD_TYPE,
            &self.owner_pubky,
            self.revision,
            &self.created_at,
            &self.updated_at,
        )?;
        validate_entity_id(&self.review_id, "reviewId")?;
        validate_pubky(&self.subject_pubky, "subjectPubky")?;
        validate_pubky(&self.listing_owner_pubky, "listingOwnerPubky")?;
        validate_entity_id(&self.listing_id, "listingId")?;

        self.ratings.validate()?;

        let text_length = self.text.chars().count();
        if !(1..=MAX_REVIEW_TEXT_LENGTH).contains(&text_length) {
            return Err(format!(
                "Validation Error: review text must be 1-{MAX_REVIEW_TEXT_LENGTH} characters"
            ));
        }

        let attestation_length = self.eligibility_attestation.chars().count();
        if !(MIN_ATTESTATION_LENGTH..=MAX_ATTESTATION_LENGTH).contains(&attestation_length) {
            return Err(format!(
                "Validation Error: eligibilityAttestation must be {MIN_ATTESTATION_LENGTH}-{MAX_ATTESTATION_LENGTH} characters"
            ));
        }
        if !self
            .eligibility_attestation
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._~-".contains(c))
        {
            return Err(
                "Validation Error: eligibilityAttestation must only contain characters [A-Za-z0-9._~-]"
                    .into(),
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::Validatable;

    const REVIEWER: &str = "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo";
    const SELLER: &str = "pxnu33x7jtpx9ar1ytsi4yxbp6a5o36gwhffs8zoxmbuptici1jy";

    fn valid_review() -> PubkyAppMarketplaceReview {
        let mut review = PubkyAppMarketplaceReview::new(
            REVIEWER.to_string(),
            1,
            "2025-01-01T00:00:00Z".to_string(),
            "2025-01-01T00:00:00Z".to_string(),
            String::new(), // assigned below from the generated hash ID
            SELLER.to_string(),
            SELLER.to_string(),
            "0032SSN7Q4EVG".to_string(),
            PubkyAppReviewRole::BuyerReviewingSeller,
            PubkyAppReviewRatings {
                overall: 5,
                item_accuracy: Some(5),
                shipping: Some(4),
                communication: None,
            },
            "Great seller, fast shipping.".to_string(),
            "a".repeat(64),
        );
        review.review_id = review.create_id();
        review
    }

    #[test]
    fn test_create_id_is_deterministic() {
        let review = valid_review();
        assert_eq!(review.create_id(), review.create_id());

        let mut other = valid_review();
        other.role = PubkyAppReviewRole::SellerReviewingBuyer;
        assert_ne!(review.create_id(), other.create_id());

        let mut other_listing = valid_review();
        other_listing.listing_id = "0032SSN7Q4EVH".to_string();
        assert_ne!(review.create_id(), other_listing.create_id());
    }

    #[test]
    fn test_create_path() {
        let review = valid_review();
        let path = PubkyAppMarketplaceReview::create_path(&review.review_id);
        assert_eq!(
            path,
            format!("/pub/pubky.app/marketplace/v1/reviews/{}", review.review_id)
        );
    }

    #[test]
    fn test_validate_valid() {
        let review = valid_review();
        let id = review.review_id.clone();
        assert!(review.validate(Some(&id)).is_ok());
    }

    #[test]
    fn test_try_from_valid_roundtrip() {
        let review = valid_review();
        let id = review.review_id.clone();
        let json = serde_json::to_string(&review).unwrap();
        let parsed =
            <PubkyAppMarketplaceReview as Validatable>::try_from(json.as_bytes(), &id).unwrap();
        assert_eq!(parsed, review);
    }

    #[test]
    fn test_try_from_rejects_unknown_field() {
        let review = valid_review();
        let id = review.review_id.clone();
        let mut value = serde_json::to_value(&review).unwrap();
        value["surprise"] = serde_json::json!(true);
        let json = serde_json::to_string(&value).unwrap();
        assert!(
            <PubkyAppMarketplaceReview as Validatable>::try_from(json.as_bytes(), &id).is_err()
        );
    }

    #[test]
    fn test_validate_wrong_id() {
        let review = valid_review();
        assert!(review.validate(Some("WRONGID123456789012345678")).is_err());
    }

    #[test]
    fn test_validate_review_id_mismatch() {
        let mut review = valid_review();
        let id = review.review_id.clone();
        review.review_id = "different_id".to_string();
        // The hash is derived from target fields, not review_id, so the path
        // ID still regenerates; the mismatch against the record must fail.
        assert!(review.validate(Some(&id)).is_err());
    }

    #[test]
    fn test_validate_rating_out_of_range() {
        let mut review = valid_review();
        review.ratings.overall = 0;
        assert!(review.validate(None).is_err());

        let mut review = valid_review();
        review.ratings.shipping = Some(6);
        assert!(review.validate(None).is_err());
    }

    #[test]
    fn test_validate_empty_text_after_trim() {
        let mut review = valid_review();
        review.text = "   ".to_string();
        let review = review.sanitize();
        assert!(review.validate(None).is_err());
    }

    #[test]
    fn test_validate_text_too_long() {
        let mut review = valid_review();
        review.text = "a".repeat(MAX_REVIEW_TEXT_LENGTH + 1);
        assert!(review.validate(None).is_err());
    }

    #[test]
    fn test_validate_attestation_rules() {
        let mut review = valid_review();
        review.eligibility_attestation = "too-short".to_string();
        assert!(review.validate(None).is_err());

        let mut review = valid_review();
        review.eligibility_attestation = format!("{}!", "a".repeat(40));
        assert!(review.validate(None).is_err());
    }

    #[test]
    fn test_validate_invalid_subject() {
        let mut review = valid_review();
        review.subject_pubky = "not-a-pubky".to_string();
        assert!(review.validate(None).is_err());
    }

    #[test]
    fn test_sanitize_trims_text() {
        let mut review = valid_review();
        review.text = "  Great seller.  ".to_string();
        let review = review.sanitize();
        assert_eq!(review.text, "Great seller.");
    }
}
