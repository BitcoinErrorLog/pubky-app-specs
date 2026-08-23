use crate::{
    models::marketplace::{
        validate_base_record, validate_marketplace_uri, PubkyAppMarketplaceLocation,
        MARKETPLACE_SCHEMA_VERSION,
    },
    traits::{HasPath, Validatable},
    APP_PATH, PUBLIC_PATH,
};
use serde::{Deserialize, Serialize};
use url::Url;

#[cfg(target_arch = "wasm32")]
use crate::traits::Json;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "openapi")]
use utoipa::ToSchema;

// Validation (mirroring the pubky-app commerce shop record schema)
const SHOP_RECORD_TYPE: &str = "shop";
const MAX_SHOP_NAME_LENGTH: usize = 60;
const MAX_SHOP_BIO_LENGTH: usize = 1000;
const MAX_SHOP_POLICY_LENGTH: usize = 4000;
const MAX_TRANSACTION_SERVICE_LENGTH: usize = 300;

/// Represents a seller's marketplace shop profile (singleton per user).
///
/// URI: /pub/pubky.app/marketplace/v1/shop.json
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct PubkyAppShop {
    /// Marketplace contract version, always `1`.
    pub schema_version: i64,
    /// Record discriminator, always `"shop"`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub record_type: String,
    /// z-base-32 pubky of the shop owner.
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub name: String,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub bio: String,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub location: PubkyAppMarketplaceLocation,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner_url: Option<String>,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub shipping_policy: String,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    pub return_policy: String,
    pub vacation_mode: bool,
    /// HTTPS base URL of the marketplace transaction service this shop
    /// sells through. When present, clients MUST resolve transactional
    /// commands (checkout, offers, bids, orders) for this shop against this
    /// authority, falling back to their configured default when absent. Two
    /// different services may register the same public listing — the shop
    /// record is the seller's declaration of which one is authoritative.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_service: Option<String>,
}

impl PubkyAppShop {
    /// Creates a new `PubkyAppShop` instance and sanitizes it.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        owner_pubky: String,
        revision: i64,
        created_at: String,
        updated_at: String,
        name: String,
        bio: String,
        location: PubkyAppMarketplaceLocation,
        avatar_url: Option<String>,
        banner_url: Option<String>,
        shipping_policy: String,
        return_policy: String,
        vacation_mode: bool,
        transaction_service: Option<String>,
    ) -> Self {
        Self {
            schema_version: MARKETPLACE_SCHEMA_VERSION,
            record_type: SHOP_RECORD_TYPE.to_string(),
            owner_pubky,
            revision,
            created_at,
            updated_at,
            name,
            bio,
            location,
            avatar_url,
            banner_url,
            shipping_policy,
            return_policy,
            vacation_mode,
            transaction_service,
        }
        .sanitize()
    }
}

#[cfg(target_arch = "wasm32")]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl PubkyAppShop {
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = fromJson))]
    pub fn from_json(js_value: &JsValue) -> Result<Self, String> {
        Self::import_json(js_value)
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = toJson))]
    pub fn to_json(&self) -> Result<JsValue, String> {
        self.export_json()
    }

    /// Getter for `name`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Getter for `bio`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn bio(&self) -> String {
        self.bio.clone()
    }

    /// Getter for `owner_pubky`.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter))]
    pub fn owner_pubky(&self) -> String {
        self.owner_pubky.clone()
    }
}

#[cfg(target_arch = "wasm32")]
impl Json for PubkyAppShop {}

/// Validates a transaction-service authority: an HTTPS base URL with no
/// credentials, query, or fragment, at most 300 characters.
fn validate_transaction_service(value: &str) -> Result<(), String> {
    if value.chars().count() > MAX_TRANSACTION_SERVICE_LENGTH {
        return Err(format!(
            "Validation Error: transactionService must be at most {MAX_TRANSACTION_SERVICE_LENGTH} characters"
        ));
    }
    let parsed = Url::parse(value)
        .map_err(|_| "Validation Error: transactionService must be a valid URL".to_string())?;
    if parsed.scheme() != "https" {
        return Err("Validation Error: transactionService must use the https scheme".into());
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err("Validation Error: transactionService must not contain credentials".into());
    }
    if parsed.query().is_some() {
        return Err("Validation Error: transactionService must not contain a query".into());
    }
    if parsed.fragment().is_some() {
        return Err("Validation Error: transactionService must not contain a fragment".into());
    }
    Ok(())
}

impl HasPath for PubkyAppShop {
    const PATH_SEGMENT: &'static str = "marketplace/v1/shop.json";

    fn create_path() -> String {
        [PUBLIC_PATH, APP_PATH, Self::PATH_SEGMENT].concat()
    }
}

impl Validatable for PubkyAppShop {
    fn sanitize(self) -> Self {
        PubkyAppShop {
            name: self.name.trim().to_string(),
            bio: self.bio.trim().to_string(),
            location: self.location.sanitize(),
            shipping_policy: self.shipping_policy.trim().to_string(),
            return_policy: self.return_policy.trim().to_string(),
            ..self
        }
    }

    fn validate(&self, _id: Option<&str>) -> Result<(), String> {
        validate_base_record(
            self.schema_version,
            &self.record_type,
            SHOP_RECORD_TYPE,
            &self.owner_pubky,
            self.revision,
            &self.created_at,
            &self.updated_at,
        )?;

        let name_length = self.name.chars().count();
        if !(1..=MAX_SHOP_NAME_LENGTH).contains(&name_length) {
            return Err(format!(
                "Validation Error: shop name must be 1-{MAX_SHOP_NAME_LENGTH} characters"
            ));
        }

        if self.bio.chars().count() > MAX_SHOP_BIO_LENGTH {
            return Err(format!(
                "Validation Error: shop bio exceeds maximum length of {MAX_SHOP_BIO_LENGTH}"
            ));
        }

        self.location.validate(None)?;

        if let Some(avatar_url) = &self.avatar_url {
            validate_marketplace_uri(avatar_url, "avatarUrl")?;
        }
        if let Some(banner_url) = &self.banner_url {
            validate_marketplace_uri(banner_url, "bannerUrl")?;
        }

        if self.shipping_policy.chars().count() > MAX_SHOP_POLICY_LENGTH {
            return Err(format!(
                "Validation Error: shippingPolicy exceeds maximum length of {MAX_SHOP_POLICY_LENGTH}"
            ));
        }
        if self.return_policy.chars().count() > MAX_SHOP_POLICY_LENGTH {
            return Err(format!(
                "Validation Error: returnPolicy exceeds maximum length of {MAX_SHOP_POLICY_LENGTH}"
            ));
        }

        if let Some(transaction_service) = &self.transaction_service {
            validate_transaction_service(transaction_service)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::Validatable;

    const OWNER: &str = "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo";

    fn valid_shop() -> PubkyAppShop {
        PubkyAppShop::new(
            OWNER.to_string(),
            1,
            "2025-01-01T00:00:00Z".to_string(),
            "2025-01-02T00:00:00Z".to_string(),
            "Boots & Co".to_string(),
            "Quality hiking boots.".to_string(),
            PubkyAppMarketplaceLocation {
                country_code: "US".to_string(),
                region: Some("Oregon".to_string()),
            },
            Some(format!(
                "pubky://{OWNER}/pub/pubky.app/marketplace/v1/media/avatar_01"
            )),
            None,
            "Ships within 3 business days.".to_string(),
            "Returns accepted within 30 days.".to_string(),
            false,
            None,
        )
    }

    #[test]
    fn test_create_path() {
        assert_eq!(
            PubkyAppShop::create_path(),
            "/pub/pubky.app/marketplace/v1/shop.json"
        );
    }

    #[test]
    fn test_validate_valid() {
        assert!(valid_shop().validate(None).is_ok());
    }

    #[test]
    fn test_sanitize_trims_strings() {
        let shop = PubkyAppShop::new(
            OWNER.to_string(),
            1,
            "2025-01-01T00:00:00Z".to_string(),
            "2025-01-02T00:00:00Z".to_string(),
            "  Boots & Co  ".to_string(),
            "  bio  ".to_string(),
            PubkyAppMarketplaceLocation {
                country_code: "US".to_string(),
                region: Some("  Oregon  ".to_string()),
            },
            None,
            None,
            "  ship  ".to_string(),
            "  return  ".to_string(),
            true,
            None,
        );
        assert_eq!(shop.name, "Boots & Co");
        assert_eq!(shop.bio, "bio");
        assert_eq!(shop.location.region.as_deref(), Some("Oregon"));
        assert_eq!(shop.shipping_policy, "ship");
        assert_eq!(shop.return_policy, "return");
    }

    #[test]
    fn test_try_from_valid_roundtrip() {
        let shop = valid_shop();
        let json = serde_json::to_string(&shop).unwrap();
        let parsed = <PubkyAppShop as Validatable>::try_from(json.as_bytes(), "").unwrap();
        assert_eq!(parsed, shop);
    }

    #[test]
    fn test_try_from_rejects_unknown_field() {
        let shop = valid_shop();
        let mut value = serde_json::to_value(&shop).unwrap();
        value["surprise"] = serde_json::json!(true);
        let json = serde_json::to_string(&value).unwrap();
        assert!(<PubkyAppShop as Validatable>::try_from(json.as_bytes(), "").is_err());
    }

    #[test]
    fn test_validate_empty_name() {
        let mut shop = valid_shop();
        shop.name = "   ".to_string();
        let shop = shop.sanitize();
        assert!(shop.validate(None).is_err());
    }

    #[test]
    fn test_validate_name_too_long() {
        let mut shop = valid_shop();
        shop.name = "a".repeat(MAX_SHOP_NAME_LENGTH + 1);
        assert!(shop.validate(None).is_err());
    }

    #[test]
    fn test_validate_bio_too_long() {
        let mut shop = valid_shop();
        shop.bio = "a".repeat(MAX_SHOP_BIO_LENGTH + 1);
        assert!(shop.validate(None).is_err());
    }

    #[test]
    fn test_validate_policy_too_long() {
        let mut shop = valid_shop();
        shop.shipping_policy = "a".repeat(MAX_SHOP_POLICY_LENGTH + 1);
        assert!(shop.validate(None).is_err());
    }

    #[test]
    fn test_validate_invalid_country() {
        let mut shop = valid_shop();
        shop.location.country_code = "usa".to_string();
        assert!(shop.validate(None).is_err());
    }

    #[test]
    fn test_validate_invalid_avatar_uri() {
        let mut shop = valid_shop();
        shop.avatar_url = Some("https://example.com/avatar.png".to_string());
        assert!(shop.validate(None).is_err());
    }

    #[test]
    fn test_validate_dates_out_of_order() {
        let mut shop = valid_shop();
        shop.created_at = "2025-02-01T00:00:00Z".to_string();
        shop.updated_at = "2025-01-01T00:00:00Z".to_string();
        assert!(shop.validate(None).is_err());
    }

    #[test]
    fn test_validate_wrong_record_type() {
        let mut shop = valid_shop();
        shop.record_type = "listing".to_string();
        assert!(shop.validate(None).is_err());
    }

    #[test]
    fn test_validate_invalid_owner() {
        let mut shop = valid_shop();
        shop.owner_pubky = "not-a-pubky".to_string();
        assert!(shop.validate(None).is_err());
    }

    #[test]
    fn test_transaction_service_valid_https_url() {
        let mut shop = valid_shop();
        shop.transaction_service = Some("https://tx.example.com".to_string());
        assert!(shop.validate(None).is_ok());

        shop.transaction_service = Some("https://tx.example.com/api/v1".to_string());
        assert!(shop.validate(None).is_ok());
    }

    #[test]
    fn test_transaction_service_rejects_http() {
        let mut shop = valid_shop();
        shop.transaction_service = Some("http://tx.example.com".to_string());
        assert!(shop.validate(None).is_err());
    }

    #[test]
    fn test_transaction_service_rejects_query_fragment_userinfo() {
        let mut shop = valid_shop();
        shop.transaction_service = Some("https://tx.example.com/?a=1".to_string());
        assert!(shop.validate(None).is_err());

        shop.transaction_service = Some("https://tx.example.com/#section".to_string());
        assert!(shop.validate(None).is_err());

        shop.transaction_service = Some("https://user@tx.example.com".to_string());
        assert!(shop.validate(None).is_err());

        shop.transaction_service = Some("https://user:pass@tx.example.com".to_string());
        assert!(shop.validate(None).is_err());
    }

    #[test]
    fn test_transaction_service_rejects_too_long() {
        let mut shop = valid_shop();
        shop.transaction_service = Some(format!("https://tx.example.com/{}", "a".repeat(300)));
        assert!(shop.validate(None).is_err());
    }

    #[test]
    fn test_transaction_service_rejects_not_a_url() {
        let mut shop = valid_shop();
        shop.transaction_service = Some("not a url".to_string());
        assert!(shop.validate(None).is_err());
    }

    #[test]
    fn test_transaction_service_absent_when_none() {
        // Byte-compatibility with `.6` records: None must not serialize.
        let shop = valid_shop();
        let value = serde_json::to_value(&shop).unwrap();
        assert!(value.get("transactionService").is_none());
    }

    #[test]
    fn test_parses_marketplace_6_shaped_shop_without_field() {
        // A `.6`-era shop JSON (no transactionService) must keep parsing.
        let json = format!(
            r#"{{
                "schemaVersion": 1,
                "recordType": "shop",
                "ownerPubky": "{OWNER}",
                "revision": 1,
                "createdAt": "2025-01-01T00:00:00Z",
                "updatedAt": "2025-01-02T00:00:00Z",
                "name": "Boots & Co",
                "bio": "Quality hiking boots.",
                "location": {{ "countryCode": "US", "region": "Oregon" }},
                "shippingPolicy": "Ships within 3 business days.",
                "returnPolicy": "Returns accepted within 30 days.",
                "vacationMode": false
            }}"#
        );
        let parsed = <PubkyAppShop as Validatable>::try_from(json.as_bytes(), "").unwrap();
        assert_eq!(parsed.transaction_service, None);
        // And round-trips without gaining the field.
        let value = serde_json::to_value(&parsed).unwrap();
        assert!(value.get("transactionService").is_none());
    }

    #[test]
    fn test_transaction_service_roundtrip_when_present() {
        let mut shop = valid_shop();
        shop.transaction_service = Some("https://tx.example.com".to_string());
        let json = serde_json::to_string(&shop).unwrap();
        assert!(json.contains("\"transactionService\":\"https://tx.example.com\""));
        let parsed = <PubkyAppShop as Validatable>::try_from(json.as_bytes(), "").unwrap();
        assert_eq!(parsed, shop);
    }
}
