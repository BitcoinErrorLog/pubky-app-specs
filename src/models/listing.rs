use crate::{
    models::marketplace::{
        marketplace_media_prefix, parse_rfc3339_millis, validate_base_record, validate_entity_id,
        validate_hex_hash, validate_locks_uri, validate_marketplace_uri,
        PubkyAppMarketplaceLocation, PubkyAppMoney, MARKETPLACE_SCHEMA_VERSION, MAX_SAFE_INTEGER,
    },
    traits::{HasIdPath, TimestampId, Validatable},
    APP_PATH, PUBLIC_PATH,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

#[cfg(target_arch = "wasm32")]
use crate::traits::Json;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

// Validation (mirroring the pubky-app commerce listing record schema)
const LISTING_RECORD_TYPE: &str = "listing";
const MIN_TAXONOMY_VERSION: i64 = 1;
const MAX_TAXONOMY_VERSION: i64 = 1_000_000;
const MIN_TITLE_LENGTH: usize = 3;
const MAX_TITLE_LENGTH: usize = 80;
const MAX_DESCRIPTION_LENGTH: usize = 10_000;
const MAX_CATEGORY_ID_LENGTH: usize = 120;
const MAX_CONDITION_DETAILS_LENGTH: usize = 1_000;
const MAX_TAGS: usize = 10;
const MAX_TAG_LENGTH: usize = 40;
const MAX_IMAGES: usize = 12;
const MAX_VIDEOS: usize = 1;
const MAX_MEDIA: usize = MAX_IMAGES + MAX_VIDEOS;
const MAX_ALT_TEXT_LENGTH: usize = 300;
const MAX_VARIANTS: usize = 100;
const MAX_OPTION_DIMENSIONS: usize = 3;
const MAX_OPTION_KEY_LENGTH: usize = 40;
const MAX_OPTION_VALUE_LENGTH: usize = 80;
const MAX_SKU_LENGTH: usize = 64;
const MAX_QUANTITY: i64 = 1_000_000;
const MAX_FULFILLMENT_METHODS: usize = 3;
const MAX_SHIPPING_OPTIONS: usize = 20;
const MAX_SHIPPING_LABEL_LENGTH: usize = 100;
const MAX_SHIPPING_PROVIDER_LENGTH: usize = 50;
const MAX_SHIPPING_SERVICE_CODE_LENGTH: usize = 100;
const MAX_DELIVERY_ESTIMATE_DAYS: i64 = 365;
const MAX_ANTI_SNIPING_SECONDS: i64 = 3_600;
const MAX_PACKAGE_WEIGHT_GRAMS: i64 = 1_000_000;
const MAX_PACKAGE_DIMENSION_MILLIMETERS: i64 = 100_000;
const MAX_RETURN_WINDOW_DAYS: i64 = 365;
const MAX_RETURN_DETAILS_LENGTH: usize = 4_000;
const MAX_MINIMUM_CONFIRMATIONS: i64 = 6;
const MAX_ATTRIBUTES: usize = 20;
const MAX_ATTRIBUTE_KEY_LENGTH: usize = 40;
const MAX_ATTRIBUTE_VALUE_LENGTH: usize = 80;
const MAX_ATTRIBUTE_VALUES_PER_KEY: usize = 10;

fn default_true() -> bool {
    true
}

fn default_criterion_id() -> String {
    "criterion-1".to_string()
}

/// Lifecycle state of a marketplace listing.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum PubkyAppListingState {
    Active,
    Paused,
    Ended,
    Removed,
}

/// Physical condition of the listed item.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum PubkyAppListingCondition {
    New,
    LikeNew,
    Excellent,
    Good,
    Fair,
    ForParts,
}

/// How the item can be delivered to the buyer.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum PubkyAppFulfillmentMethod {
    Physical,
    Digital,
    Pickup,
}

/// Media attachment type.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum PubkyAppListingMediaKind {
    Image,
    Video,
}

/// A media attachment (image or video) referencing a marketplace media URI
/// owned by the listing seller.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PubkyAppListingMedia {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: PubkyAppListingMediaKind,
    pub url: String,
    pub content_hash: String,
    pub mime_type: String,
    pub byte_size: i64,
    pub width: i64,
    pub height: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<i64>,
    pub alt_text: String,
}

impl PubkyAppListingMedia {
    fn sanitize(self) -> Self {
        PubkyAppListingMedia {
            alt_text: self.alt_text.trim().to_string(),
            ..self
        }
    }

    fn validate(&self) -> Result<(), String> {
        validate_entity_id(&self.id, "media id")?;
        validate_marketplace_uri(&self.url, "media url")?;
        validate_hex_hash(&self.content_hash, "media contentHash")?;
        validate_mime_type(&self.mime_type)?;
        if !(1..=MAX_SAFE_INTEGER).contains(&self.byte_size) {
            return Err("Validation Error: media byteSize must be a positive safe integer".into());
        }
        if !(1..=MAX_SAFE_INTEGER).contains(&self.width)
            || !(1..=MAX_SAFE_INTEGER).contains(&self.height)
        {
            return Err(
                "Validation Error: media width and height must be positive integers".into(),
            );
        }
        match (self.kind, self.duration_ms) {
            (PubkyAppListingMediaKind::Video, None) => {
                return Err("Validation Error: video media requires durationMs".into())
            }
            (PubkyAppListingMediaKind::Image, Some(_)) => {
                return Err("Validation Error: image media cannot declare durationMs".into())
            }
            (PubkyAppListingMediaKind::Video, Some(duration_ms))
                if !(1..=MAX_SAFE_INTEGER).contains(&duration_ms) =>
            {
                return Err("Validation Error: media durationMs must be a positive integer".into())
            }
            _ => (),
        }
        let alt_text_length = self.alt_text.chars().count();
        if !(1..=MAX_ALT_TEXT_LENGTH).contains(&alt_text_length) {
            return Err(format!(
                "Validation Error: media altText must be 1-{MAX_ALT_TEXT_LENGTH} characters"
            ));
        }
        Ok(())
    }
}

fn validate_mime_type(value: &str) -> Result<(), String> {
    let error = "Validation Error: media mimeType must be an image or video MIME type".to_string();
    let (prefix, subtype) = value.split_once('/').ok_or_else(|| error.clone())?;
    if !prefix.eq_ignore_ascii_case("image") && !prefix.eq_ignore_ascii_case("video") {
        return Err(error);
    }
    if subtype.is_empty()
        || !subtype
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || ".+-".contains(c))
    {
        return Err(error);
    }
    Ok(())
}

/// A purchasable variant (SKU) of a listing.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PubkyAppListingVariant {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sku: Option<String>,
    pub options: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_override: Option<PubkyAppMoney>,
    pub quantity: i64,
    #[serde(default)]
    pub media_ids: Vec<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl PubkyAppListingVariant {
    fn sanitize(self) -> Self {
        PubkyAppListingVariant {
            sku: self.sku.map(|sku| sku.trim().to_string()),
            options: self
                .options
                .into_iter()
                .map(|(key, value)| (key.trim().to_string(), value.trim().to_string()))
                .collect(),
            ..self
        }
    }

    fn validate(&self) -> Result<(), String> {
        validate_entity_id(&self.id, "variant id")?;
        if let Some(sku) = &self.sku {
            let length = sku.chars().count();
            if !(1..=MAX_SKU_LENGTH).contains(&length) {
                return Err(format!(
                    "Validation Error: variant sku must be 1-{MAX_SKU_LENGTH} characters"
                ));
            }
        }
        if self.options.len() > MAX_OPTION_DIMENSIONS {
            return Err(format!(
                "Validation Error: variants support at most {MAX_OPTION_DIMENSIONS} option dimensions"
            ));
        }
        for (key, value) in &self.options {
            let key_length = key.chars().count();
            let value_length = value.chars().count();
            if !(1..=MAX_OPTION_KEY_LENGTH).contains(&key_length) {
                return Err(format!(
                    "Validation Error: variant option keys must be 1-{MAX_OPTION_KEY_LENGTH} characters"
                ));
            }
            if !(1..=MAX_OPTION_VALUE_LENGTH).contains(&value_length) {
                return Err(format!(
                    "Validation Error: variant option values must be 1-{MAX_OPTION_VALUE_LENGTH} characters"
                ));
            }
        }
        if let Some(price_override) = &self.price_override {
            price_override.validate_positive("variant priceOverride")?;
        }
        if !(0..=MAX_QUANTITY).contains(&self.quantity) {
            return Err(format!(
                "Validation Error: variant quantity must be between 0 and {MAX_QUANTITY}"
            ));
        }
        if self.media_ids.len() > MAX_MEDIA {
            return Err(format!(
                "Validation Error: variant mediaIds exceeds maximum of {MAX_MEDIA}"
            ));
        }
        for media_id in &self.media_ids {
            validate_entity_id(media_id, "variant mediaIds entry")?;
        }
        Ok(())
    }
}

/// How the listing is sold: fixed price or auction.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "format", rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum PubkyAppListingSale {
    #[serde(rename_all = "camelCase")]
    FixedPrice {
        unit_price: PubkyAppMoney,
        accepts_offers: bool,
    },
    #[serde(rename_all = "camelCase")]
    Auction {
        starting_price: PubkyAppMoney,
        #[serde(skip_serializing_if = "Option::is_none")]
        reserve_price: Option<PubkyAppMoney>,
        #[serde(skip_serializing_if = "Option::is_none")]
        buy_now_price: Option<PubkyAppMoney>,
        minimum_increment: PubkyAppMoney,
        starts_at: String,
        ends_at: String,
        anti_sniping_window_seconds: i64,
        anti_sniping_extension_seconds: i64,
    },
}

impl PubkyAppListingSale {
    /// The price that defines the listing's asset (currency + exponent).
    pub fn primary_price(&self) -> &PubkyAppMoney {
        match self {
            PubkyAppListingSale::FixedPrice { unit_price, .. } => unit_price,
            PubkyAppListingSale::Auction { starting_price, .. } => starting_price,
        }
    }

    fn validate(&self) -> Result<(), String> {
        match self {
            PubkyAppListingSale::FixedPrice { unit_price, .. } => {
                unit_price.validate_positive("sale unitPrice")
            }
            PubkyAppListingSale::Auction {
                starting_price,
                reserve_price,
                buy_now_price,
                minimum_increment,
                starts_at,
                ends_at,
                anti_sniping_window_seconds,
                anti_sniping_extension_seconds,
            } => {
                starting_price.validate_positive("sale startingPrice")?;
                minimum_increment.validate_positive("sale minimumIncrement")?;
                let starts = parse_rfc3339_millis(starts_at)
                    .map_err(|e| format!("Validation Error: sale startsAt {e}"))?;
                let ends = parse_rfc3339_millis(ends_at)
                    .map_err(|e| format!("Validation Error: sale endsAt {e}"))?;
                if ends <= starts {
                    return Err("Validation Error: auction end must follow its start".into());
                }
                if !(0..=MAX_ANTI_SNIPING_SECONDS).contains(anti_sniping_window_seconds)
                    || !(0..=MAX_ANTI_SNIPING_SECONDS).contains(anti_sniping_extension_seconds)
                {
                    return Err(format!(
                        "Validation Error: anti-sniping windows must be between 0 and {MAX_ANTI_SNIPING_SECONDS} seconds"
                    ));
                }
                if !minimum_increment.same_asset(starting_price) {
                    return Err(
                        "Validation Error: auction prices must use one asset and exponent".into(),
                    );
                }
                if let Some(reserve_price) = reserve_price {
                    reserve_price.validate_positive("sale reservePrice")?;
                    if !reserve_price.same_asset(starting_price) {
                        return Err(
                            "Validation Error: auction prices must use one asset and exponent"
                                .into(),
                        );
                    }
                    if reserve_price.amount_minor < starting_price.amount_minor {
                        return Err(
                            "Validation Error: reserve price must not be below the starting price"
                                .into(),
                        );
                    }
                }
                if let Some(buy_now_price) = buy_now_price {
                    buy_now_price.validate_positive("sale buyNowPrice")?;
                    if !buy_now_price.same_asset(starting_price) {
                        return Err(
                            "Validation Error: auction prices must use one asset and exponent"
                                .into(),
                        );
                    }
                    if buy_now_price.amount_minor <= starting_price.amount_minor {
                        return Err(
                            "Validation Error: buy-now price must exceed the starting price".into(),
                        );
                    }
                }
                Ok(())
            }
        }
    }
}

/// A delivery option offered by the seller.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "pricing", rename_all = "snake_case")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum PubkyAppShippingOption {
    #[serde(rename_all = "camelCase")]
    Free {
        id: String,
        label: String,
        estimated_min_days: i64,
        estimated_max_days: i64,
    },
    #[serde(rename_all = "camelCase")]
    Flat {
        id: String,
        label: String,
        price: PubkyAppMoney,
        estimated_min_days: i64,
        estimated_max_days: i64,
    },
    #[serde(rename_all = "camelCase")]
    Calculated {
        id: String,
        label: String,
        provider: String,
        service_code: String,
        estimated_min_days: i64,
        estimated_max_days: i64,
    },
}

impl PubkyAppShippingOption {
    /// The unique identifier of this shipping option.
    pub fn id(&self) -> &str {
        match self {
            PubkyAppShippingOption::Free { id, .. }
            | PubkyAppShippingOption::Flat { id, .. }
            | PubkyAppShippingOption::Calculated { id, .. } => id,
        }
    }

    fn sanitize(self) -> Self {
        match self {
            PubkyAppShippingOption::Free {
                id,
                label,
                estimated_min_days,
                estimated_max_days,
            } => PubkyAppShippingOption::Free {
                id,
                label: label.trim().to_string(),
                estimated_min_days,
                estimated_max_days,
            },
            PubkyAppShippingOption::Flat {
                id,
                label,
                price,
                estimated_min_days,
                estimated_max_days,
            } => PubkyAppShippingOption::Flat {
                id,
                label: label.trim().to_string(),
                price,
                estimated_min_days,
                estimated_max_days,
            },
            PubkyAppShippingOption::Calculated {
                id,
                label,
                provider,
                service_code,
                estimated_min_days,
                estimated_max_days,
            } => PubkyAppShippingOption::Calculated {
                id,
                label: label.trim().to_string(),
                provider: provider.trim().to_string(),
                service_code: service_code.trim().to_string(),
                estimated_min_days,
                estimated_max_days,
            },
        }
    }

    fn validate(&self) -> Result<(), String> {
        let (label, min_days, max_days) = match self {
            PubkyAppShippingOption::Free {
                id,
                label,
                estimated_min_days,
                estimated_max_days,
            } => {
                validate_entity_id(id, "shipping option id")?;
                (label, *estimated_min_days, *estimated_max_days)
            }
            PubkyAppShippingOption::Flat {
                id,
                label,
                price,
                estimated_min_days,
                estimated_max_days,
            } => {
                validate_entity_id(id, "shipping option id")?;
                price.validate_non_negative("shipping price")?;
                (label, *estimated_min_days, *estimated_max_days)
            }
            PubkyAppShippingOption::Calculated {
                id,
                label,
                provider,
                service_code,
                estimated_min_days,
                estimated_max_days,
            } => {
                validate_entity_id(id, "shipping option id")?;
                let provider_length = provider.chars().count();
                if !(1..=MAX_SHIPPING_PROVIDER_LENGTH).contains(&provider_length) {
                    return Err(format!(
                        "Validation Error: shipping provider must be 1-{MAX_SHIPPING_PROVIDER_LENGTH} characters"
                    ));
                }
                let service_code_length = service_code.chars().count();
                if !(1..=MAX_SHIPPING_SERVICE_CODE_LENGTH).contains(&service_code_length) {
                    return Err(format!(
                        "Validation Error: shipping serviceCode must be 1-{MAX_SHIPPING_SERVICE_CODE_LENGTH} characters"
                    ));
                }
                (label, *estimated_min_days, *estimated_max_days)
            }
        };

        let label_length = label.chars().count();
        if !(1..=MAX_SHIPPING_LABEL_LENGTH).contains(&label_length) {
            return Err(format!(
                "Validation Error: shipping label must be 1-{MAX_SHIPPING_LABEL_LENGTH} characters"
            ));
        }
        if !(0..=MAX_DELIVERY_ESTIMATE_DAYS).contains(&min_days)
            || !(0..=MAX_DELIVERY_ESTIMATE_DAYS).contains(&max_days)
        {
            return Err(format!(
                "Validation Error: delivery estimates must be between 0 and {MAX_DELIVERY_ESTIMATE_DAYS} days"
            ));
        }
        if max_days < min_days {
            return Err(
                "Validation Error: maximum delivery estimate must not precede the minimum".into(),
            );
        }
        Ok(())
    }
}

/// Package facts required for physically fulfilled listings.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PubkyAppListingPackage {
    pub weight_grams: i64,
    pub length_millimeters: i64,
    pub width_millimeters: i64,
    pub height_millimeters: i64,
}

impl PubkyAppListingPackage {
    fn validate(&self) -> Result<(), String> {
        if !(1..=MAX_PACKAGE_WEIGHT_GRAMS).contains(&self.weight_grams) {
            return Err(format!(
                "Validation Error: package weightGrams must be between 1 and {MAX_PACKAGE_WEIGHT_GRAMS}"
            ));
        }
        for dimension in [
            self.length_millimeters,
            self.width_millimeters,
            self.height_millimeters,
        ] {
            if !(1..=MAX_PACKAGE_DIMENSION_MILLIMETERS).contains(&dimension) {
                return Err(format!(
                    "Validation Error: package dimensions must be between 1 and {MAX_PACKAGE_DIMENSION_MILLIMETERS} millimeters"
                ));
            }
        }
        Ok(())
    }
}

/// The seller's return policy for a listing.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PubkyAppReturnPolicy {
    pub accepts_returns: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_window_days: Option<i64>,
    pub buyer_pays_return_shipping: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl PubkyAppReturnPolicy {
    fn sanitize(self) -> Self {
        PubkyAppReturnPolicy {
            details: self.details.map(|details| details.trim().to_string()),
            ..self
        }
    }

    fn validate(&self) -> Result<(), String> {
        match (self.accepts_returns, self.return_window_days) {
            (true, None) => {
                return Err(
                    "Validation Error: a return window is required when returns are accepted"
                        .into(),
                )
            }
            (false, Some(_)) => {
                return Err(
                    "Validation Error: a return window cannot be set when returns are not accepted"
                        .into(),
                )
            }
            (true, Some(days)) if !(1..=MAX_RETURN_WINDOW_DAYS).contains(&days) => {
                return Err(format!(
                    "Validation Error: returnWindowDays must be between 1 and {MAX_RETURN_WINDOW_DAYS}"
                ))
            }
            _ => (),
        }
        if let Some(details) = &self.details {
            if details.chars().count() > MAX_RETURN_DETAILS_LENGTH {
                return Err(format!(
                    "Validation Error: return policy details exceed maximum length of {MAX_RETURN_DETAILS_LENGTH}"
                ));
            }
        }
        Ok(())
    }
}

/// Locks policy configuration required for digitally fulfilled listings.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PubkyAppDigitalLock {
    pub policy_uri: String,
    #[serde(default = "default_criterion_id")]
    pub criterion_id: String,
    pub resource_hash: String,
    pub minimum_confirmations: i64,
}

impl PubkyAppDigitalLock {
    fn validate(&self) -> Result<(), String> {
        validate_locks_uri(&self.policy_uri, "digitalLock policyUri")?;
        validate_entity_id(&self.criterion_id, "digitalLock criterionId")?;
        validate_hex_hash(&self.resource_hash, "digitalLock resourceHash")?;
        if !(0..=MAX_MINIMUM_CONFIRMATIONS).contains(&self.minimum_confirmations) {
            return Err(format!(
                "Validation Error: digitalLock minimumConfirmations must be between 0 and {MAX_MINIMUM_CONFIRMATIONS}"
            ));
        }
        Ok(())
    }
}

/// One item-specific attribute value: a single string or a small list of
/// strings (e.g. up to two colors). Serialized untagged, so records carry
/// plain JSON strings or arrays of strings.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum PubkyAppListingAttributeValue {
    One(String),
    Many(Vec<String>),
}

impl PubkyAppListingAttributeValue {
    fn sanitize(self) -> Self {
        match self {
            PubkyAppListingAttributeValue::One(value) => {
                PubkyAppListingAttributeValue::One(value.trim().to_string())
            }
            PubkyAppListingAttributeValue::Many(values) => PubkyAppListingAttributeValue::Many(
                values.into_iter().map(|v| v.trim().to_string()).collect(),
            ),
        }
    }

    fn validate(&self, key: &str) -> Result<(), String> {
        let validate_entry = |value: &str| -> Result<(), String> {
            let length = value.chars().count();
            if !(1..=MAX_ATTRIBUTE_VALUE_LENGTH).contains(&length) {
                return Err(format!(
                    "Validation Error: attribute '{key}' values must be 1-{MAX_ATTRIBUTE_VALUE_LENGTH} characters"
                ));
            }
            Ok(())
        };
        match self {
            PubkyAppListingAttributeValue::One(value) => validate_entry(value),
            PubkyAppListingAttributeValue::Many(values) => {
                if !(1..=MAX_ATTRIBUTE_VALUES_PER_KEY).contains(&values.len()) {
                    return Err(format!(
                        "Validation Error: attribute '{key}' supports 1-{MAX_ATTRIBUTE_VALUES_PER_KEY} values"
                    ));
                }
                for value in values {
                    validate_entry(value)?;
                }
                ensure_unique(
                    values.iter().map(String::as_str),
                    &format!("attribute '{key}' values"),
                )
            }
        }
    }
}

/// Attribute keys are lowercase alphanumeric identifiers with single `-` or
/// `_` separators (e.g. `size`, `color`, `age-era`).
fn validate_attribute_key(key: &str) -> Result<(), String> {
    let length = key.chars().count();
    if !(1..=MAX_ATTRIBUTE_KEY_LENGTH).contains(&length) {
        return Err(format!(
            "Validation Error: attribute keys must be 1-{MAX_ATTRIBUTE_KEY_LENGTH} characters"
        ));
    }
    let all_parts_valid = key.split(['-', '_']).all(|part| {
        !part.is_empty()
            && part
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
    });
    if !all_parts_valid {
        return Err(
            "Validation Error: attribute keys must be lowercase alphanumeric identifiers".into(),
        );
    }
    Ok(())
}

/// Represents a marketplace listing published by a seller.
///
/// URI: /pub/pubky.app/marketplace/v1/listings/:listing_id
///
/// Where listing_id is Crockford-base32 encoding of a timestamp and must
/// match the record's `listingId` field.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PubkyAppListing {
    /// Marketplace contract version, always `1`.
    pub schema_version: i64,
    /// Record discriminator, always `"listing"`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub record_type: String,
    /// z-base-32 pubky of the seller.
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
    /// Must match the timestamp ID in the record's path.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub listing_id: String,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub state: PubkyAppListingState,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub title: String,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub description: String,
    /// Marketplace category taxonomy version, `1` or greater. The category
    /// tree and the attribute expectations per category are versioned CLIENT
    /// configuration keyed by this number — the record stays self-describing
    /// without the spec churning per category.
    pub taxonomy_version: i64,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub category_id: String,
    /// Item specifics: a bounded, generic key/value container. Which keys a
    /// category expects (and their allowed values) is client configuration
    /// keyed by `taxonomy_version`; the spec only enforces shape bounds.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attributes: Option<BTreeMap<String, PubkyAppListingAttributeValue>>,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub condition: PubkyAppListingCondition,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition_details: Option<String>,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub tags: Vec<String>,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub location: PubkyAppMarketplaceLocation,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub media: Vec<PubkyAppListingMedia>,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub variants: Vec<PubkyAppListingVariant>,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub sale: PubkyAppListingSale,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub fulfillment_methods: Vec<PubkyAppFulfillmentMethod>,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package: Option<PubkyAppListingPackage>,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub shipping_options: Vec<PubkyAppShippingOption>,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub return_policy: PubkyAppReturnPolicy,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub digital_lock: Option<PubkyAppDigitalLock>,
    pub adult_only: bool,
}

impl PubkyAppListing {
    /// Creates a new `PubkyAppListing` instance and sanitizes it.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        owner_pubky: String,
        revision: i64,
        created_at: String,
        updated_at: String,
        listing_id: String,
        state: PubkyAppListingState,
        title: String,
        description: String,
        taxonomy_version: i64,
        category_id: String,
        attributes: Option<BTreeMap<String, PubkyAppListingAttributeValue>>,
        condition: PubkyAppListingCondition,
        condition_details: Option<String>,
        tags: Vec<String>,
        location: PubkyAppMarketplaceLocation,
        media: Vec<PubkyAppListingMedia>,
        variants: Vec<PubkyAppListingVariant>,
        sale: PubkyAppListingSale,
        fulfillment_methods: Vec<PubkyAppFulfillmentMethod>,
        package: Option<PubkyAppListingPackage>,
        shipping_options: Vec<PubkyAppShippingOption>,
        return_policy: PubkyAppReturnPolicy,
        digital_lock: Option<PubkyAppDigitalLock>,
        adult_only: bool,
    ) -> Self {
        Self {
            schema_version: MARKETPLACE_SCHEMA_VERSION,
            record_type: LISTING_RECORD_TYPE.to_string(),
            owner_pubky,
            revision,
            created_at,
            updated_at,
            listing_id,
            state,
            title,
            description,
            taxonomy_version,
            category_id,
            attributes,
            condition,
            condition_details,
            tags,
            location,
            media,
            variants,
            sale,
            fulfillment_methods,
            package,
            shipping_options,
            return_policy,
            digital_lock,
            adult_only,
        }
        .sanitize()
    }
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl PubkyAppListing {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = fromJson))]
    pub fn from_json(js_value: &JsValue) -> Result<Self, String> {
        Self::import_json(js_value)
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = toJson))]
    pub fn to_json(&self) -> Result<JsValue, String> {
        self.export_json()
    }

    /// Getter for `title`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn title(&self) -> String {
        self.title.clone()
    }

    /// Getter for `description`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn description(&self) -> String {
        self.description.clone()
    }

    /// Getter for `listing_id`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn listing_id(&self) -> String {
        self.listing_id.clone()
    }

    /// Getter for `owner_pubky`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn owner_pubky(&self) -> String {
        self.owner_pubky.clone()
    }
}

#[cfg(target_arch = "wasm32")]
impl Json for PubkyAppListing {}

impl TimestampId for PubkyAppListing {}

impl HasIdPath for PubkyAppListing {
    const PATH_SEGMENT: &'static str = "marketplace/v1/listings/";

    fn create_path(id: &str) -> String {
        [PUBLIC_PATH, APP_PATH, Self::PATH_SEGMENT, id].concat()
    }
}

fn validate_kebab_case_category(value: &str) -> Result<(), String> {
    let length = value.chars().count();
    if !(1..=MAX_CATEGORY_ID_LENGTH).contains(&length) {
        return Err(format!(
            "Validation Error: categoryId must be 1-{MAX_CATEGORY_ID_LENGTH} characters"
        ));
    }
    let all_parts_valid = value.split('-').all(|part| {
        !part.is_empty()
            && part
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
    });
    if !all_parts_valid {
        return Err("Validation Error: categoryId must be a kebab-case identifier".into());
    }
    Ok(())
}

fn ensure_unique<'a, I: IntoIterator<Item = &'a str>>(values: I, what: &str) -> Result<(), String> {
    let mut seen = HashSet::new();
    for value in values {
        if !seen.insert(value) {
            return Err(format!("Validation Error: {what} must be unique"));
        }
    }
    Ok(())
}

impl Validatable for PubkyAppListing {
    fn sanitize(self) -> Self {
        PubkyAppListing {
            title: self.title.trim().to_string(),
            description: self.description.trim().to_string(),
            condition_details: self
                .condition_details
                .map(|details| details.trim().to_string()),
            attributes: self.attributes.map(|attributes| {
                attributes
                    .into_iter()
                    .map(|(key, value)| (key, value.sanitize()))
                    .collect()
            }),
            tags: self
                .tags
                .into_iter()
                .map(|tag| tag.trim().to_string())
                .collect(),
            location: self.location.sanitize(),
            media: self
                .media
                .into_iter()
                .map(PubkyAppListingMedia::sanitize)
                .collect(),
            variants: self
                .variants
                .into_iter()
                .map(PubkyAppListingVariant::sanitize)
                .collect(),
            shipping_options: self
                .shipping_options
                .into_iter()
                .map(PubkyAppShippingOption::sanitize)
                .collect(),
            return_policy: self.return_policy.sanitize(),
            ..self
        }
    }

    fn validate(&self, id: Option<&str>) -> Result<(), String> {
        // Validate the listing path ID and its match with the record. The path
        // ID follows the marketplace entity-id convention (path-safe, bounded)
        // rather than the 13-char Crockford timestamp rule: the reference
        // client and the transaction service key listings by 32-char lowercase
        // hex UUIDs, and every canonical record on homeservers carries one.
        // Builder-generated timestamp IDs (see `TimestampId::create_id`, still
        // implemented for this type) satisfy the same rule, so both forms
        // validate. Requiring timestamp IDs here made every real listing
        // unindexable (found 2026-08-21 when Nexus rejected all listing PUTs
        // with "Invalid ID length: must be 13 characters").
        if let Some(id) = id {
            validate_entity_id(id, "listing path id")?;
            if self.listing_id != id {
                return Err(
                    "Validation Error: listingId does not match the listing path ID".into(),
                );
            }
        }

        validate_base_record(
            self.schema_version,
            &self.record_type,
            LISTING_RECORD_TYPE,
            &self.owner_pubky,
            self.revision,
            &self.created_at,
            &self.updated_at,
        )?;
        validate_entity_id(&self.listing_id, "listingId")?;

        if !(MIN_TAXONOMY_VERSION..=MAX_TAXONOMY_VERSION).contains(&self.taxonomy_version) {
            return Err(format!(
                "Validation Error: taxonomyVersion must be between {MIN_TAXONOMY_VERSION} and {MAX_TAXONOMY_VERSION}"
            ));
        }

        let title_length = self.title.chars().count();
        if !(MIN_TITLE_LENGTH..=MAX_TITLE_LENGTH).contains(&title_length) {
            return Err(format!(
                "Validation Error: listing title must be {MIN_TITLE_LENGTH}-{MAX_TITLE_LENGTH} characters"
            ));
        }
        let description_length = self.description.chars().count();
        if !(1..=MAX_DESCRIPTION_LENGTH).contains(&description_length) {
            return Err(format!(
                "Validation Error: listing description must be 1-{MAX_DESCRIPTION_LENGTH} characters"
            ));
        }

        validate_kebab_case_category(&self.category_id)?;

        if let Some(attributes) = &self.attributes {
            if attributes.len() > MAX_ATTRIBUTES {
                return Err(format!(
                    "Validation Error: listing supports at most {MAX_ATTRIBUTES} attributes"
                ));
            }
            for (key, value) in attributes {
                validate_attribute_key(key)?;
                value.validate(key)?;
            }
        }

        if let Some(condition_details) = &self.condition_details {
            if condition_details.chars().count() > MAX_CONDITION_DETAILS_LENGTH {
                return Err(format!(
                    "Validation Error: conditionDetails exceeds maximum length of {MAX_CONDITION_DETAILS_LENGTH}"
                ));
            }
        }

        if self.tags.len() > MAX_TAGS {
            return Err(format!(
                "Validation Error: listing supports at most {MAX_TAGS} tags"
            ));
        }
        for tag in &self.tags {
            let tag_length = tag.chars().count();
            if !(1..=MAX_TAG_LENGTH).contains(&tag_length) {
                return Err(format!(
                    "Validation Error: listing tags must be 1-{MAX_TAG_LENGTH} characters"
                ));
            }
        }
        ensure_unique(self.tags.iter().map(String::as_str), "tags")?;

        self.location.validate(None)?;

        // Media
        if !(1..=MAX_MEDIA).contains(&self.media.len()) {
            return Err(format!(
                "Validation Error: listing requires 1-{MAX_MEDIA} media entries"
            ));
        }
        let media_prefix = marketplace_media_prefix(&self.owner_pubky);
        for media in &self.media {
            media.validate()?;
            if !media.url.starts_with(&media_prefix) {
                return Err(
                    "Validation Error: listing media must be owned by the listing seller".into(),
                );
            }
        }
        ensure_unique(
            self.media.iter().map(|media| media.id.as_str()),
            "media ids",
        )?;
        let image_count = self
            .media
            .iter()
            .filter(|media| media.kind == PubkyAppListingMediaKind::Image)
            .count();
        let video_count = self.media.len() - image_count;
        if !(1..=MAX_IMAGES).contains(&image_count) {
            return Err(format!(
                "Validation Error: listings require 1-{MAX_IMAGES} images"
            ));
        }
        if video_count > MAX_VIDEOS {
            return Err(format!(
                "Validation Error: listings support at most {MAX_VIDEOS} video"
            ));
        }

        // Variants
        if !(1..=MAX_VARIANTS).contains(&self.variants.len()) {
            return Err(format!(
                "Validation Error: listing requires 1-{MAX_VARIANTS} variants"
            ));
        }
        for variant in &self.variants {
            variant.validate()?;
        }
        ensure_unique(
            self.variants.iter().map(|variant| variant.id.as_str()),
            "variant ids",
        )?;
        ensure_unique(
            self.variants
                .iter()
                .filter_map(|variant| variant.sku.as_deref()),
            "variant SKUs",
        )?;
        let media_ids: HashSet<&str> = self.media.iter().map(|media| media.id.as_str()).collect();
        for variant in &self.variants {
            for media_id in &variant.media_ids {
                if !media_ids.contains(media_id.as_str()) {
                    return Err("Validation Error: variant references unknown media".into());
                }
            }
        }
        if self.state == PubkyAppListingState::Active
            && !self
                .variants
                .iter()
                .any(|variant| variant.enabled && variant.quantity > 0)
        {
            return Err(
                "Validation Error: an active listing requires available intended quantity".into(),
            );
        }

        // Sale
        self.sale.validate()?;
        if matches!(self.sale, PubkyAppListingSale::Auction { .. }) && self.variants.len() != 1 {
            return Err("Validation Error: auction listings require exactly one variant".into());
        }
        let primary_price = self.sale.primary_price();
        for variant in &self.variants {
            if let Some(price_override) = &variant.price_override {
                if !price_override.same_asset(primary_price) {
                    return Err(
                        "Validation Error: variant price must use the listing asset and exponent"
                            .into(),
                    );
                }
            }
        }

        // Fulfillment
        if !(1..=MAX_FULFILLMENT_METHODS).contains(&self.fulfillment_methods.len()) {
            return Err(format!(
                "Validation Error: listing requires 1-{MAX_FULFILLMENT_METHODS} fulfillment methods"
            ));
        }
        let unique_methods: HashSet<&PubkyAppFulfillmentMethod> =
            self.fulfillment_methods.iter().collect();
        if unique_methods.len() != self.fulfillment_methods.len() {
            return Err("Validation Error: fulfillment methods must be unique".into());
        }

        // Shipping
        if self.shipping_options.len() > MAX_SHIPPING_OPTIONS {
            return Err(format!(
                "Validation Error: listing supports at most {MAX_SHIPPING_OPTIONS} shipping options"
            ));
        }
        for option in &self.shipping_options {
            option.validate()?;
            if let PubkyAppShippingOption::Flat { price, .. } = option {
                if !price.same_asset(primary_price) {
                    return Err(
                        "Validation Error: shipping price must use the listing asset and exponent"
                            .into(),
                    );
                }
            }
        }
        ensure_unique(
            self.shipping_options.iter().map(|option| option.id()),
            "shipping option ids",
        )?;

        self.return_policy.validate()?;

        if let Some(digital_lock) = &self.digital_lock {
            digital_lock.validate()?;
        }

        // Fulfillment cross-field rules
        let has_physical = self
            .fulfillment_methods
            .contains(&PubkyAppFulfillmentMethod::Physical);
        let has_digital = self
            .fulfillment_methods
            .contains(&PubkyAppFulfillmentMethod::Digital);
        if has_physical && self.package.is_none() {
            return Err("Validation Error: physical fulfillment requires package facts".into());
        }
        if has_physical && self.shipping_options.is_empty() {
            return Err("Validation Error: physical fulfillment requires a shipping option".into());
        }
        if !has_physical && (self.package.is_some() || !self.shipping_options.is_empty()) {
            return Err(
                "Validation Error: package and shipping options require physical fulfillment"
                    .into(),
            );
        }
        if has_digital != self.digital_lock.is_some() {
            return Err(
                "Validation Error: digital fulfillment and digitalLock must be configured together"
                    .into(),
            );
        }

        if let Some(package) = &self.package {
            package.validate()?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::Validatable;

    const OWNER: &str = "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo";

    fn money(amount_minor: i64) -> PubkyAppMoney {
        PubkyAppMoney {
            amount_minor,
            currency: "USD".to_string(),
            exponent: 2,
        }
    }

    fn valid_media(id: &str) -> PubkyAppListingMedia {
        PubkyAppListingMedia {
            id: id.to_string(),
            kind: PubkyAppListingMediaKind::Image,
            url: format!("pubky://{OWNER}/pub/pubky.app/marketplace/v1/media/{id}"),
            content_hash: "a".repeat(64),
            mime_type: "image/png".to_string(),
            byte_size: 1024,
            width: 800,
            height: 600,
            duration_ms: None,
            alt_text: "A pair of boots".to_string(),
        }
    }

    fn valid_variant(id: &str) -> PubkyAppListingVariant {
        PubkyAppListingVariant {
            id: id.to_string(),
            sku: Some(format!("SKU-{id}")),
            options: BTreeMap::from([("size".to_string(), "42".to_string())]),
            price_override: None,
            quantity: 5,
            media_ids: vec![],
            enabled: true,
        }
    }

    fn valid_listing() -> PubkyAppListing {
        let listing = PubkyAppListing::new(
            OWNER.to_string(),
            1,
            "2025-01-01T00:00:00Z".to_string(),
            "2025-01-02T00:00:00Z".to_string(),
            String::new(), // assigned below from the generated timestamp ID
            PubkyAppListingState::Active,
            "Hiking boots".to_string(),
            "Sturdy leather hiking boots.".to_string(),
            1,
            "fashion".to_string(),
            None,
            PubkyAppListingCondition::New,
            None,
            vec!["boots".to_string(), "hiking".to_string()],
            PubkyAppMarketplaceLocation {
                country_code: "US".to_string(),
                region: Some("Oregon".to_string()),
            },
            vec![valid_media("image_01")],
            vec![valid_variant("variant_01")],
            PubkyAppListingSale::FixedPrice {
                unit_price: money(12_000),
                accepts_offers: true,
            },
            vec![PubkyAppFulfillmentMethod::Physical],
            Some(PubkyAppListingPackage {
                weight_grams: 1_500,
                length_millimeters: 350,
                width_millimeters: 250,
                height_millimeters: 150,
            }),
            vec![PubkyAppShippingOption::Flat {
                id: "ship_01".to_string(),
                label: "Standard".to_string(),
                price: money(500),
                estimated_min_days: 2,
                estimated_max_days: 7,
            }],
            PubkyAppReturnPolicy {
                accepts_returns: true,
                return_window_days: Some(30),
                buyer_pays_return_shipping: true,
                details: None,
            },
            None,
            false,
        );
        let mut listing = listing;
        listing.listing_id = listing.create_id();
        listing
    }

    fn valid_auction_listing() -> PubkyAppListing {
        let mut listing = valid_listing();
        listing.sale = PubkyAppListingSale::Auction {
            starting_price: money(1_000),
            reserve_price: Some(money(2_000)),
            buy_now_price: Some(money(10_000)),
            minimum_increment: money(100),
            starts_at: "2025-01-03T00:00:00Z".to_string(),
            ends_at: "2025-01-10T00:00:00Z".to_string(),
            anti_sniping_window_seconds: 300,
            anti_sniping_extension_seconds: 300,
        };
        listing
    }

    #[test]
    fn test_create_path() {
        let listing = valid_listing();
        let path = PubkyAppListing::create_path(&listing.listing_id);
        assert_eq!(
            path,
            format!(
                "/pub/pubky.app/marketplace/v1/listings/{}",
                listing.listing_id
            )
        );
    }

    #[test]
    fn test_validate_valid_fixed_price() {
        let listing = valid_listing();
        let id = listing.listing_id.clone();
        assert!(listing.validate(Some(&id)).is_ok());
    }

    #[test]
    fn test_validate_valid_auction() {
        let listing = valid_auction_listing();
        let id = listing.listing_id.clone();
        assert!(listing.validate(Some(&id)).is_ok());
    }

    #[test]
    fn test_try_from_valid_roundtrip() {
        let listing = valid_listing();
        let id = listing.listing_id.clone();
        let json = serde_json::to_string(&listing).unwrap();
        let parsed = <PubkyAppListing as Validatable>::try_from(json.as_bytes(), &id).unwrap();
        assert_eq!(parsed, listing);
    }

    #[test]
    fn test_serialized_field_names_are_camel_case() {
        let listing = valid_listing();
        let value = serde_json::to_value(&listing).unwrap();
        assert_eq!(value["recordType"], "listing");
        assert_eq!(value["schemaVersion"], 1);
        assert!(value["media"][0].get("altText").is_some());
        assert!(value["media"][0].get("type").is_some());
        assert_eq!(value["sale"]["format"], "fixed_price");
        assert!(value["sale"].get("unitPrice").is_some());
        assert_eq!(value["shippingOptions"][0]["pricing"], "flat");
        // Absent optionals are omitted, not null
        assert!(value.get("conditionDetails").is_none());
    }

    #[test]
    fn test_try_from_accepts_unknown_field() {
        let listing = valid_listing();
        let id = listing.listing_id.clone();
        let mut value = serde_json::to_value(&listing).unwrap();
        value["surprise"] = serde_json::json!(true);
        let json = serde_json::to_string(&value).unwrap();
        assert!(<PubkyAppListing as Validatable>::try_from(json.as_bytes(), &id).is_ok());
    }

    #[test]
    fn test_validate_invalid_timestamp_id() {
        let listing = valid_listing();
        assert!(listing.validate(Some("INVALIDID12345")).is_err());
    }

    #[test]
    fn test_validate_listing_id_mismatch() {
        let mut listing = valid_listing();
        let id = listing.listing_id.clone();
        listing.listing_id = "different_id".to_string();
        assert!(listing.validate(Some(&id)).is_err());
    }

    #[test]
    fn test_validate_accepts_uuid_hex_path_id() {
        // The reference client and transaction service key listings by
        // 32-char lowercase hex UUIDs; the path-id rule must accept them.
        let mut listing = valid_listing();
        let id = "a7fc7d5d0b2a4083b27847193f8fe536".to_string();
        listing.listing_id = id.clone();
        assert!(listing.validate(Some(&id)).is_ok());
    }

    #[test]
    fn test_validate_accepts_builder_timestamp_path_id() {
        // Builder-generated 13-char Crockford ids stay valid under the
        // entity-id rule.
        let mut listing = valid_listing();
        let id = listing.create_id();
        listing.listing_id = id.clone();
        assert!(listing.validate(Some(&id)).is_ok());
    }

    #[test]
    fn test_validate_title_too_short() {
        let mut listing = valid_listing();
        listing.title = "ab".to_string();
        assert!(listing.validate(None).is_err());
    }

    #[test]
    fn test_validate_taxonomy_version_bounds() {
        let mut listing = valid_listing();
        listing.taxonomy_version = 2;
        assert!(listing.validate(None).is_ok());

        listing.taxonomy_version = 0;
        assert!(listing.validate(None).is_err());
        listing.taxonomy_version = MAX_TAXONOMY_VERSION + 1;
        assert!(listing.validate(None).is_err());
    }

    fn attributes(
        entries: &[(&str, PubkyAppListingAttributeValue)],
    ) -> Option<BTreeMap<String, PubkyAppListingAttributeValue>> {
        Some(
            entries
                .iter()
                .map(|(key, value)| (key.to_string(), value.clone()))
                .collect(),
        )
    }

    #[test]
    fn test_validate_valid_attributes() {
        let mut listing = valid_listing();
        listing.attributes = attributes(&[
            (
                "size",
                PubkyAppListingAttributeValue::One("US 9".to_string()),
            ),
            (
                "color",
                PubkyAppListingAttributeValue::Many(vec!["brown".to_string(), "black".to_string()]),
            ),
            (
                "age-era",
                PubkyAppListingAttributeValue::One("90s".to_string()),
            ),
        ]);
        assert!(listing.validate(None).is_ok());
    }

    #[test]
    fn test_validate_attributes_absent_is_valid() {
        let mut listing = valid_listing();
        listing.attributes = None;
        assert!(listing.validate(None).is_ok());
    }

    #[test]
    fn test_validate_attribute_key_charset() {
        for bad_key in ["Size", "size color", "-size", "size-", "size--color", ""] {
            let mut listing = valid_listing();
            listing.attributes = attributes(&[(
                bad_key,
                PubkyAppListingAttributeValue::One("value".to_string()),
            )]);
            assert!(
                listing.validate(None).is_err(),
                "expected key {bad_key:?} to be rejected"
            );
        }
    }

    #[test]
    fn test_validate_attribute_bounds() {
        // Too many keys
        let mut listing = valid_listing();
        let entries: Vec<(String, PubkyAppListingAttributeValue)> = (0..=MAX_ATTRIBUTES)
            .map(|index| {
                (
                    format!("key-{index}"),
                    PubkyAppListingAttributeValue::One("value".to_string()),
                )
            })
            .collect();
        listing.attributes = Some(entries.into_iter().collect());
        assert!(listing.validate(None).is_err());

        // Value too long
        let mut listing = valid_listing();
        listing.attributes = attributes(&[(
            "size",
            PubkyAppListingAttributeValue::One("x".repeat(MAX_ATTRIBUTE_VALUE_LENGTH + 1)),
        )]);
        assert!(listing.validate(None).is_err());

        // Empty value
        let mut listing = valid_listing();
        listing.attributes =
            attributes(&[("size", PubkyAppListingAttributeValue::One(String::new()))]);
        assert!(listing.validate(None).is_err());

        // Too many values per key
        let mut listing = valid_listing();
        listing.attributes = attributes(&[(
            "style",
            PubkyAppListingAttributeValue::Many(
                (0..=MAX_ATTRIBUTE_VALUES_PER_KEY)
                    .map(|index| format!("value-{index}"))
                    .collect(),
            ),
        )]);
        assert!(listing.validate(None).is_err());

        // Duplicate values in a list
        let mut listing = valid_listing();
        listing.attributes = attributes(&[(
            "color",
            PubkyAppListingAttributeValue::Many(vec!["brown".to_string(), "brown".to_string()]),
        )]);
        assert!(listing.validate(None).is_err());

        // Empty value list
        let mut listing = valid_listing();
        listing.attributes = attributes(&[("color", PubkyAppListingAttributeValue::Many(vec![]))]);
        assert!(listing.validate(None).is_err());
    }

    #[test]
    fn test_attributes_serialize_untagged_and_roundtrip() {
        let mut listing = valid_listing();
        listing.attributes = attributes(&[
            (
                "size",
                PubkyAppListingAttributeValue::One("US 9".to_string()),
            ),
            (
                "color",
                PubkyAppListingAttributeValue::Many(vec!["brown".to_string(), "black".to_string()]),
            ),
        ]);
        let id = listing.listing_id.clone();
        let value = serde_json::to_value(&listing).unwrap();
        assert_eq!(value["attributes"]["size"], "US 9");
        assert_eq!(
            value["attributes"]["color"],
            serde_json::json!(["brown", "black"])
        );
        let json = serde_json::to_string(&value).unwrap();
        let parsed = <PubkyAppListing as Validatable>::try_from(json.as_bytes(), &id).unwrap();
        assert_eq!(parsed, listing);
    }

    #[test]
    fn test_attributes_sanitize_trims_values() {
        let mut listing = valid_listing();
        listing.attributes = attributes(&[
            (
                "size",
                PubkyAppListingAttributeValue::One("  US 9  ".to_string()),
            ),
            (
                "color",
                PubkyAppListingAttributeValue::Many(vec!["  brown  ".to_string()]),
            ),
        ]);
        let sanitized = listing.sanitize();
        let attributes = sanitized.attributes.unwrap();
        assert_eq!(
            attributes.get("size"),
            Some(&PubkyAppListingAttributeValue::One("US 9".to_string()))
        );
        assert_eq!(
            attributes.get("color"),
            Some(&PubkyAppListingAttributeValue::Many(vec![
                "brown".to_string()
            ]))
        );
    }

    #[test]
    fn test_validate_invalid_category() {
        let mut listing = valid_listing();
        listing.category_id = "Not-Kebab".to_string();
        assert!(listing.validate(None).is_err());
        listing.category_id = "double--dash".to_string();
        assert!(listing.validate(None).is_err());
    }

    #[test]
    fn test_validate_duplicate_tags() {
        let mut listing = valid_listing();
        listing.tags = vec!["boots".to_string(), "boots".to_string()];
        assert!(listing.validate(None).is_err());
    }

    #[test]
    fn test_validate_duplicate_variant_ids() {
        let mut listing = valid_listing();
        listing.variants = vec![valid_variant("variant_01"), valid_variant("variant_01")];
        assert!(listing.validate(None).is_err());
    }

    #[test]
    fn test_validate_media_not_owned_by_seller() {
        let mut listing = valid_listing();
        listing.media[0].url =
            "pubky://pxnu33x7jtpx9ar1ytsi4yxbp6a5o36gwhffs8zoxmbuptici1jy/pub/pubky.app/marketplace/v1/media/image_01"
                .to_string();
        assert!(listing.validate(None).is_err());
    }

    #[test]
    fn test_validate_video_requires_duration() {
        let mut listing = valid_listing();
        let mut video = valid_media("video_01");
        video.kind = PubkyAppListingMediaKind::Video;
        video.mime_type = "video/mp4".to_string();
        video.duration_ms = None;
        listing.media.push(video);
        assert!(listing.validate(None).is_err());
    }

    #[test]
    fn test_validate_requires_at_least_one_image() {
        let mut listing = valid_listing();
        let mut video = valid_media("video_01");
        video.kind = PubkyAppListingMediaKind::Video;
        video.mime_type = "video/mp4".to_string();
        video.duration_ms = Some(10_000);
        listing.media = vec![video];
        assert!(listing.validate(None).is_err());
    }

    #[test]
    fn test_validate_variant_unknown_media_reference() {
        let mut listing = valid_listing();
        listing.variants[0].media_ids = vec!["missing_media".to_string()];
        assert!(listing.validate(None).is_err());
    }

    #[test]
    fn test_validate_active_listing_requires_stock() {
        let mut listing = valid_listing();
        listing.variants[0].quantity = 0;
        assert!(listing.validate(None).is_err());

        listing.state = PubkyAppListingState::Paused;
        assert!(listing.validate(None).is_ok());
    }

    #[test]
    fn test_validate_negative_quantity() {
        let mut listing = valid_listing();
        listing.variants[0].quantity = -1;
        assert!(listing.validate(None).is_err());
    }

    #[test]
    fn test_validate_physical_requires_package_and_shipping() {
        let mut listing = valid_listing();
        listing.package = None;
        assert!(listing.validate(None).is_err());

        let mut listing = valid_listing();
        listing.shipping_options = vec![];
        assert!(listing.validate(None).is_err());
    }

    #[test]
    fn test_validate_pickup_forbids_package_and_shipping() {
        let mut listing = valid_listing();
        listing.fulfillment_methods = vec![PubkyAppFulfillmentMethod::Pickup];
        assert!(listing.validate(None).is_err());

        listing.package = None;
        listing.shipping_options = vec![];
        assert!(listing.validate(None).is_ok());
    }

    #[test]
    fn test_validate_digital_requires_lock() {
        let mut listing = valid_listing();
        listing.fulfillment_methods = vec![PubkyAppFulfillmentMethod::Digital];
        listing.package = None;
        listing.shipping_options = vec![];
        assert!(listing.validate(None).is_err());

        listing.digital_lock = Some(PubkyAppDigitalLock {
            policy_uri: format!("pubky://{OWNER}/pub/locks.app/policies/standard.json"),
            criterion_id: "criterion-1".to_string(),
            resource_hash: "b".repeat(64),
            minimum_confirmations: 2,
        });
        assert!(listing.validate(None).is_ok());
    }

    #[test]
    fn test_validate_auction_requires_single_variant() {
        let mut listing = valid_auction_listing();
        listing.variants = vec![valid_variant("variant_01"), valid_variant("variant_02")];
        assert!(listing.validate(None).is_err());
    }

    #[test]
    fn test_validate_auction_price_rules() {
        let mut listing = valid_auction_listing();
        if let PubkyAppListingSale::Auction { buy_now_price, .. } = &mut listing.sale {
            *buy_now_price = Some(money(1_000)); // equal to starting price
        }
        assert!(listing.validate(None).is_err());

        let mut listing = valid_auction_listing();
        if let PubkyAppListingSale::Auction { reserve_price, .. } = &mut listing.sale {
            *reserve_price = Some(money(500)); // below starting price
        }
        assert!(listing.validate(None).is_err());

        let mut listing = valid_auction_listing();
        if let PubkyAppListingSale::Auction {
            starts_at, ends_at, ..
        } = &mut listing.sale
        {
            *starts_at = "2025-01-10T00:00:00Z".to_string();
            *ends_at = "2025-01-03T00:00:00Z".to_string();
        }
        assert!(listing.validate(None).is_err());
    }

    #[test]
    fn test_validate_variant_price_must_match_asset() {
        let mut listing = valid_listing();
        listing.variants[0].price_override = Some(PubkyAppMoney {
            amount_minor: 21,
            currency: "BTC".to_string(),
            exponent: 8,
        });
        assert!(listing.validate(None).is_err());
    }

    #[test]
    fn test_validate_flat_shipping_must_match_asset() {
        let mut listing = valid_listing();
        listing.shipping_options = vec![PubkyAppShippingOption::Flat {
            id: "ship_01".to_string(),
            label: "Standard".to_string(),
            price: PubkyAppMoney {
                amount_minor: 21,
                currency: "BTC".to_string(),
                exponent: 8,
            },
            estimated_min_days: 2,
            estimated_max_days: 7,
        }];
        assert!(listing.validate(None).is_err());
    }

    #[test]
    fn test_validate_shipping_estimate_order() {
        let mut listing = valid_listing();
        listing.shipping_options = vec![PubkyAppShippingOption::Free {
            id: "ship_01".to_string(),
            label: "Free".to_string(),
            estimated_min_days: 7,
            estimated_max_days: 2,
        }];
        assert!(listing.validate(None).is_err());
    }

    #[test]
    fn test_validate_return_policy_window_rules() {
        let mut listing = valid_listing();
        listing.return_policy.return_window_days = None;
        assert!(listing.validate(None).is_err());

        let mut listing = valid_listing();
        listing.return_policy.accepts_returns = false;
        assert!(listing.validate(None).is_err());
    }

    #[test]
    fn test_sanitize_trims_strings() {
        let mut listing = valid_listing();
        listing.title = "  Hiking boots  ".to_string();
        listing.tags = vec!["  boots  ".to_string()];
        listing.variants[0].sku = Some("  SKU-1  ".to_string());
        listing.variants[0].options =
            BTreeMap::from([("  size  ".to_string(), "  42  ".to_string())]);
        let sanitized = listing.sanitize();
        assert_eq!(sanitized.title, "Hiking boots");
        assert_eq!(sanitized.tags, vec!["boots".to_string()]);
        assert_eq!(sanitized.variants[0].sku.as_deref(), Some("SKU-1"));
        assert_eq!(
            sanitized.variants[0]
                .options
                .get("size")
                .map(String::as_str),
            Some("42")
        );
    }

    #[test]
    fn test_variant_defaults_apply_on_deserialize() {
        let json = r#"{
            "id": "variant_01",
            "options": {"size": "42"},
            "quantity": 5
        }"#;
        let variant: PubkyAppListingVariant = serde_json::from_str(json).unwrap();
        assert!(variant.enabled);
        assert!(variant.media_ids.is_empty());
        assert!(variant.validate().is_ok());
    }
}
