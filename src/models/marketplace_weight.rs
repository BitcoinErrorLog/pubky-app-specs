//! The price weight of a counted review (counting rules v2): a 1-5 weight
//! from the price decade the purchase attestation signs, used by the
//! "weighted by purchase value" rating and the "highest-value purchases
//! first" sort.
//!
//! The attestation's `price_decade` claim is `"{CURRENCY}:{k}"` with
//! `k = floor(log10(unit price paid in minor units))`, the encoding of
//! `amount_band`. The weight is `w = clamp(k + offset(CURRENCY) - 1, 1, 5)`,
//! where the offset turns the currency's minor-unit decade into a US-cent
//! decade: under $10 weighs 1, $10-99 weighs 2, $100-999 weighs 3,
//! $1,000-9,999 weighs 4 and $10,000 or more weighs 5. An attestation
//! without the claim, or with a currency the table lacks, weighs 1.

/// Lowest price weight; also the weight of a missing claim or an unknown
/// currency.
pub const MIN_PRICE_WEIGHT: u8 = 1;
/// Highest price weight.
pub const MAX_PRICE_WEIGHT: u8 = 5;

/// i64 minor units cap a decade at 18.
const MAX_DECADE_MAGNITUDE: u32 = 18;

/// US-cent decade offset per currency code, for decades of that currency's
/// minor unit. The table is static and normative: every indexer and
/// recompute tool derives the same weight from the same claim, so it does
/// not follow exchange rates.
///
/// - `USD`, `EUR`, `GBP`, `CHF` (exponent 2): one minor unit is within a
///   decade of one US cent, offset 0.
/// - `SAT` (exponent 0): one satoshi is (BTC/USD) / 10^6 US cents, offset -1,
///   the nearest decade while one bitcoin trades between about $31,600 and
///   $316,000.
pub const PRICE_WEIGHT_DECADE_OFFSETS: &[(&str, i32)] =
    &[("USD", 0), ("EUR", 0), ("GBP", 0), ("CHF", 0), ("SAT", -1)];

/// The decade offset of `currency`, or `None` when the table has no entry.
pub fn price_weight_decade_offset(currency: &str) -> Option<i32> {
    PRICE_WEIGHT_DECADE_OFFSETS
        .iter()
        .find(|(code, _)| *code == currency)
        .map(|(_, offset)| *offset)
}

/// The `price_decade` claim value an issuer writes for a unit price of
/// `unit_price_minor` minor units of `currency`, e.g. `("USD", 4_999)` ->
/// `"USD:3"`. `None` for a non-positive price: there is no decade to sign,
/// so the claim is left out and the review weighs 1.
pub fn price_decade(currency: &str, unit_price_minor: i64) -> Option<String> {
    if unit_price_minor < 1 {
        return None;
    }
    Some(format!("{currency}:{}", unit_price_minor.ilog10()))
}

/// The price weight of a review whose attestation carries `price_decade`
/// (`None` when the claim is absent, which weighs 1). Errors when the value
/// is not a `{CURRENCY}:{magnitude}` decade.
pub fn price_weight(price_decade: Option<&str>) -> Result<u8, String> {
    let Some(value) = price_decade else {
        return Ok(MIN_PRICE_WEIGHT);
    };
    let (currency, magnitude) = parse_log_decade(value, "price_decade")?;
    let Some(offset) = price_weight_decade_offset(currency) else {
        return Ok(MIN_PRICE_WEIGHT);
    };
    let decade = magnitude as i32 + offset;
    Ok((decade - 1).clamp(i32::from(MIN_PRICE_WEIGHT), i32::from(MAX_PRICE_WEIGHT)) as u8)
}

/// Parses a `{CURRENCY}:{magnitude}` log decade (`amount_band`,
/// `price_decade`): an uppercase 3-12 character asset code that starts with
/// a letter, and a magnitude of 0-18 written in at most two digits.
pub(crate) fn parse_log_decade<'a>(value: &'a str, field: &str) -> Result<(&'a str, u32), String> {
    let error =
        || format!("Validation Error: {field} must be {{CURRENCY}}:{{magnitude}}, e.g. SAT:5");
    let (currency, magnitude) = value.split_once(':').ok_or_else(error)?;
    let currency_length = currency.chars().count();
    if !(3..=12).contains(&currency_length) {
        return Err(error());
    }
    let mut chars = currency.chars();
    let first_is_upper = chars.next().is_some_and(|c| c.is_ascii_uppercase());
    if !first_is_upper || !chars.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()) {
        return Err(error());
    }
    match magnitude.parse::<u32>() {
        Ok(m) if magnitude.len() <= 2 && m <= MAX_DECADE_MAGNITUDE => Ok((currency, m)),
        _ => Err(error()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct Vector {
        currency: String,
        unit_price_minor: i64,
        price_decade: Option<String>,
        weight: u8,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Vectors {
        #[allow(dead_code)]
        description: String,
        vectors: Vec<Vector>,
    }

    #[test]
    fn test_shared_vectors() {
        let fixture: Vectors =
            serde_json::from_str(include_str!("../test_fixtures/price-weight-vectors.json"))
                .expect("vector fixture parses");
        assert!(!fixture.vectors.is_empty());
        for vector in fixture.vectors {
            let decade = price_decade(&vector.currency, vector.unit_price_minor);
            assert_eq!(
                decade, vector.price_decade,
                "{} {}",
                vector.currency, vector.unit_price_minor
            );
            assert_eq!(
                price_weight(decade.as_deref()),
                Ok(vector.weight),
                "{} {}",
                vector.currency,
                vector.unit_price_minor
            );
        }
    }

    #[test]
    fn test_us_cent_decade_edges() {
        for currency in ["USD", "EUR", "GBP", "CHF"] {
            for (unit_price_minor, weight) in [
                (1, 1),
                (999, 1),       // $9.99
                (1_000, 2),     // $10.00
                (9_999, 2),     // $99.99
                (10_000, 3),    // $100.00
                (99_999, 3),    // $999.99
                (100_000, 4),   // $1,000.00
                (999_999, 4),   // $9,999.99
                (1_000_000, 5), // $10,000.00
                (i64::MAX, 5),  // clamped
            ] {
                let decade = price_decade(currency, unit_price_minor);
                assert_eq!(
                    price_weight(decade.as_deref()),
                    Ok(weight),
                    "{currency} {unit_price_minor}"
                );
            }
        }
    }

    #[test]
    fn test_sat_decade_edges() {
        for (unit_price_minor, weight) in [
            (1, 1),
            (9_999, 1),
            (10_000, 2),
            (99_999, 2),
            (100_000, 3),
            (999_999, 3),
            (1_000_000, 4),
            (9_999_999, 4),
            (10_000_000, 5),
            (2_100_000_000_000_000, 5),
        ] {
            let decade = price_decade("SAT", unit_price_minor);
            assert_eq!(
                price_weight(decade.as_deref()),
                Ok(weight),
                "SAT {unit_price_minor}"
            );
        }
    }

    #[test]
    fn test_unknown_currency_and_missing_claim_weigh_one() {
        assert_eq!(price_weight(Some("XYZ:9")), Ok(MIN_PRICE_WEIGHT));
        assert_eq!(price_weight(Some("XYZ:0")), Ok(MIN_PRICE_WEIGHT));
        assert_eq!(price_weight(None), Ok(MIN_PRICE_WEIGHT));
        assert_eq!(price_weight_decade_offset("XYZ"), None);
    }

    #[test]
    fn test_malformed_price_decade_rejected() {
        for value in [
            "", "USD", "USD:", ":3", "usd:3", "US:3", "1USD:3", "USD:19", "USD:007", "USD:-1",
            "USD:3.0", "USD:3:4", "USD 3",
        ] {
            assert!(price_weight(Some(value)).is_err(), "{value:?}");
        }
    }

    #[test]
    fn test_non_positive_price_has_no_decade() {
        assert_eq!(price_decade("USD", 0), None);
        assert_eq!(price_decade("USD", -1_000), None);
    }

    #[test]
    fn test_offset_table() {
        assert_eq!(price_weight_decade_offset("USD"), Some(0));
        assert_eq!(price_weight_decade_offset("EUR"), Some(0));
        assert_eq!(price_weight_decade_offset("GBP"), Some(0));
        assert_eq!(price_weight_decade_offset("CHF"), Some(0));
        assert_eq!(price_weight_decade_offset("SAT"), Some(-1));
        assert_eq!(PRICE_WEIGHT_DECADE_OFFSETS.len(), 5);
    }
}
