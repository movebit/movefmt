module complex_module {

    // A simple S structure which has both copy and drop abilities
    struct S has copy, drop {
        f: bool
    }

    // Incentive parameters for assorted operations.
    struct IncentiveParameters has drop, key {
        // `Coin.value` required to register a market.
        market_registration_fee: u64,
        // `Coin.value` required to register as an underwriter.
        underwriter_registration_fee: u64,
        // `Coin.value` required to register as a custodian.
        custodian_registration_fee: u64,
        // Nominal amount divisor for quote coin fee charged to takers.
        taker_fee_divisor: u64
    }

    // Integrator fee store tier parameters for a given tier.
    struct IntegratorFeeStoreTierParameters has drop, store {
        // Nominal amount divisor for taker quote coin fee.
        fee_share_divisor: u64
    }
}
