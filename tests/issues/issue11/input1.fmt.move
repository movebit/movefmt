module test {
    fun test() {
        assert!(get_econia_fee_store_balance_test<QC>(market_id_1) == econia_fees_1, 0);
        assert!(get_econia_fee_store_balance_test<QC>(market_id_1) == econia_fees_1, 0); // Assert Econia fee share.
    }
}
