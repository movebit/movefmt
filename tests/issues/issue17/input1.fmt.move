module test {
    fun test() {
        let input_deposited = 1;
        let output_deposited = 2;
        let aaaaaaaaaaaa = 1;
        let bbbbbbbbbbbb = 2;
        let cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc = 3;
        let dddddddddddd = 4;
        let eeeeeeeeeeee = 5;

        assert!(
            coin::balance<AptosCoin>(shareholder_1_address)
                == shareholder_1_bal + pending_distribution / 4,
            0
        );
        assert!(
            coin::balance<AptosCoin>(shareholder_2_address)
                == shareholder_2_bal + pending_distribution * 3 / 4,
            1
        );

        let xxxxxxxxxxxxxxxxxxxxxxxxxxxx =
            aaaaaaaaaaaa
                + bbbbbbbbbbbb
                    * cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc
                - dddddddddddd / eeeeeeeeeeee;

        (
            (int2bv((((1 as u8) << ((feature % (8 as u64)) as u64)) as u8)) as u8)
                & features[feature / 8] as u8
        ) > (0 as u8) && (feature / 8) < len(features)
    }

    spec schema UpdateAuthKeyAndOriginatingAddressTableAbortsIf {
        let stake_balance_0 = stake_pool_res.active.value
            + stake_pool_res.pending_active.value
            + stake_pool_res.pending_inactive.value;
        let stake_balance_1 = stake_pool_res.active.value
            + stake_pool_res.pending_inactive.value;

        aborts_if table::spec_contains(address_map, curr_auth_key)
            && table::spec_get(address_map, curr_auth_key) != originating_addr;
    }
}
