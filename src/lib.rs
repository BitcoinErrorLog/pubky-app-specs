mod common;
mod constants;
pub mod limits;
mod models;
pub mod traits;
mod types;
mod uri;

// Re-export constants
pub use constants::{APP_PATH, MARKETPLACE_PATH, PRIVATE_PATH, PROTOCOL, PUBLIC_PATH, VERSION};
// Re-export common utilities
pub use common::validate_crockford_id;
#[doc(inline)]
pub use limits::*;
// Re-export domain types
pub use models::blob::PubkyAppBlob;
pub use models::bookmark::PubkyAppBookmark;
pub use models::feed::{PubkyAppFeed, PubkyAppFeedLayout, PubkyAppFeedReach, PubkyAppFeedSort};
pub use models::file::{PubkyAppFile, VALID_MIME_TYPES};
pub use models::follow::PubkyAppFollow;
pub use models::last_read::PubkyAppLastRead;
pub use models::listing::{
    PubkyAppDigitalLock, PubkyAppFulfillmentMethod, PubkyAppListing, PubkyAppListingAttributeValue,
    PubkyAppListingCondition, PubkyAppListingMedia, PubkyAppListingMediaKind,
    PubkyAppListingPackage, PubkyAppListingSale, PubkyAppListingState, PubkyAppListingVariant,
    PubkyAppReturnPolicy, PubkyAppShippingOption,
};
pub use models::marketplace::{
    PubkyAppMarketplaceLocation, PubkyAppMoney, MARKETPLACE_SCHEMA_VERSION,
};
pub use models::marketplace_attestation::{
    base64url_encode, PubkyAppPurchaseAttestation, PubkyAppPurchaseAttestationClaims,
    PURCHASE_ATTESTATION_TYP, PURCHASE_ATTESTATION_VERSION,
};
pub use models::marketplace_review::{
    PubkyAppMarketplaceReview, PubkyAppReviewRatings, PubkyAppReviewRole,
};
pub use models::mute::PubkyAppMute;
pub use models::post::{
    PubkyAppCollectionContent, PubkyAppCollectionLayout, PubkyAppPost, PubkyAppPostEmbed,
    PubkyAppPostKind,
};
pub use models::review_response::PubkyAppReviewResponse;
pub use models::shop::PubkyAppShop;
pub use models::tag::PubkyAppTag;
pub use models::user::{PubkyAppUser, PubkyAppUserLink};
pub use models::watchlist::{
    PubkyAppWatchlist, PubkyAppWatchlistItem, PubkyAppWatchlistTombstone, MAX_WATCHLIST_ITEMS,
    MAX_WATCHLIST_TOMBSTONES,
};
pub use models::PubkyAppObject;
pub use types::PubkyId;
#[doc(inline)]
pub use uri::{
    base_uri_builder, blob_uri_builder, bookmark_uri_builder, feed_uri_builder, file_uri_builder,
    follow_uri_builder, is_pubky_scheme, last_read_uri_builder, listing_uri_builder,
    marketplace_review_uri_builder, mute_uri_builder, post_uri_builder,
    review_response_uri_builder, shop_uri_builder, tag_uri_builder, try_parse_pubky_path,
    user_uri_builder, watchlist_uri_builder, ExtendedParsedUri, ParsedUri, PubkyPath, Resource,
};

// Our WASM module
#[cfg(target_arch = "wasm32")]
mod wasm;
// Re-export the Wasm functions so they're available to wasm-pack
#[cfg(target_arch = "wasm32")]
pub use wasm::*;
