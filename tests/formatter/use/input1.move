/// test_point: A list consisting of multiple items, with comments after the items

module test_use {
    use aptos_framework::aptos_coin::AptosCoin;
        use aptos_framework::coin::{Self, Coin}/* use */;
                use aptos_std::type_info::{Self/* use_item after */, TypeInfo};
            use econia::resource_account;
        use econia::tablist::{Self, Tablist/* use_item after */};
            use std::signer::address_of;
    use std::vector;
}