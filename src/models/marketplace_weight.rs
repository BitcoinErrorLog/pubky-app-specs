//! The price weight of a counted review (counting rules v2): a 1-5 weight
//! from the reviewed listing's price decade, used by the "weighted by
//! purchase value" rating and the "highest-value purchases first" sort.
//!
//! `d = floor(log10(price_minor)) + offset(currency)`, where the offset turns
//! the currency's minor-unit decade into a US-cent decade, and
//! `w = clamp(d - 1, 1, 5)`: under $10 weighs 1, $10-99 weighs 2,
//! $100-999 weighs 3, $1,000-9,999 weighs 4 and $10,000 or more weighs 5.

/// Lowest price weight; also the weight of an unknown currency or a
/// non-positive price.
pub const MIN_PRICE_WEIGHT: u8 = 1;
/// Highest price weight.
pub const MAX_PRICE_WEIGHT: u8 = 5;

/// US-cent decade offset per currency code, for prices in that currency's
/// minor unit. The table is static and normative: every indexer and
/// recompute tool must derive the same weight from the same price, so it
/// does not follow exchange rates.
///
/// - `USD`, `EUR`, `GBP`, `CHF` (exponent 2): one minor unit is within a
///   decade of one US cent, offset 0.
/// - `SAT` (exponent 0): one satoshi is BTC/USD / 10^6 US cents, offset -1,
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

/// The price weight of a purchase priced `price_minor` minor units of
/// `currency`. Unknown currencies and non-positive prices weigh 1.
pub fn price_weight(currency: &str, price_minor: i64) -> u8 {
    let Some(offset) = price_weight_decade_offset(currency) else {
        return MIN_PRICE_WEIGHT;
    };
    if price_minor <= 0 {
        return MIN_PRICE_WEIGHT;
    }
    let decade = price_minor.ilog10() as i32 + offset;
    (decade - 1).clamp(i32::from(MIN_PRICE_WEIGHT), i32::from(MAX_PRICE_WEIGHT)) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_us_cent_decade_edges() {
        for currency in ["USD", "EUR", "GBP", "CHF"] {
            for (price_minor, weight) in [
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
                assert_eq!(
                    price_weight(currency, price_minor),
                    weight,
                    "{currency} {price_minor}"
                );
            }
        }
    }

    #[test]
    fn test_sat_decade_edges() {
        for (price_minor, weight) in [
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
            assert_eq!(
                price_weight("SAT", price_minor),
                weight,
                "SAT {price_minor}"
            );
        }
    }

    #[test]
    fn test_unknown_currency_weighs_one() {
        assert_eq!(price_weight("XYZ", 1_000_000_000), MIN_PRICE_WEIGHT);
        assert_eq!(price_weight("usd", 1_000_000), MIN_PRICE_WEIGHT);
        assert_eq!(price_weight("", 1_000_000), MIN_PRICE_WEIGHT);
        assert_eq!(price_weight_decade_offset("XYZ"), None);
    }

    #[test]
    fn test_non_positive_price_weighs_one() {
        assert_eq!(price_weight("USD", 0), MIN_PRICE_WEIGHT);
        assert_eq!(price_weight("USD", -1_000_000), MIN_PRICE_WEIGHT);
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
