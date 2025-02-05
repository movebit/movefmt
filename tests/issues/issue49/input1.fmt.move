module test_binop {
    fun get_types_for_event<CoinType>(gift_asset: &GiftAsset<CoinType>): (String, bool) {
        match(gift_asset) {
            GiftAsset::Coin { coin: _ } => (type_info::type_name<CoinType>(), false),
            GiftAsset::FungibleAsset { fa_metadata } => (object::object_address(fa_metadata), true)
        }
    }
}
