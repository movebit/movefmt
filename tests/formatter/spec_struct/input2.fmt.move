/// test_point: has {'invariant'}

/// The `ASCII` module defines basic string and char newtypes in Move that verify
/// that characters are valid ASCII, and that strings consist of only valid ASCII characters.
module std::ascii {
    use std::vector;
    use std::option::{Self, Option};
    use aptos_framework::chain_status;
    use aptos_framework::coin::CoinInfo;
    use aptos_framework::aptos_coin::AptosCoin;
    use aptos_framework::transaction_fee;
    use aptos_framework::staking_config;

    /// An ASCII character.
    struct Char has copy, drop, store {
        byte: u8
    }

    spec Char {
        invariant is_valid_char(byte);
    }
}
