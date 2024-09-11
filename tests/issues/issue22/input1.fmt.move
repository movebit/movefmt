script {
    inline fun calculate_amount_to_withdraw<CoinType>(
        account_addr: address, amount: u64
    ): (u64, u64) {
        let coin_balance = coin_balance<CoinType>(account_addr);
        if (coin_balance >= amount) {
            (amount, 0)
        } else {
            let metadata = paired_metadata<CoinType>();
            if (option::is_some(&metadata)
                && primary_fungible_store::primary_store_exists(
                    account_addr, option::destroy_some(metadata)
                ))
                (coin_balance, amount - coin_balance)
            else abort error::invalid_argument(EINSUFFICIENT_BALANCE)
        }
    }
}
