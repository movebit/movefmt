module test {
    inline fun swap_within_escrow<Coin0, LP0, Coin1, LP1>(
        swapper: &signer,
        swapper_address: address,
        escrow_ref_mut: &mut Escrow<Coin0, LP0, Coin1, LP1>,
        market_address_0: address,
        market_address_1: address
    ): (u64, u64, u64, u64) {
        max_quote_match = calculate_max_quote_match(direction, taker_fee_divisor, max_quote_delta_user);
        if (true) {
            (quote_volume, integrator_fee, emojicoin_1_proceeds) = swap_within_escrow_for_direction<Coin0, LP0, Coin1, LP1>(
                swapper,
                swapper_address,
                market_address_0,
                market_address_1,
                emojicoin_0_balance,
                &mut escrow_ref_mut.emojicoin_0,
                &mut escrow_ref_mut.emojicoin_1
            );
        } else {
            // Verify that at least one emojicoin balance is nonzero.
            assert!(emojicoin_1_balance > 0, E_SWAP_NO_FUNDS);

            (quote_volume, integrator_fee, emojicoin_0_proceeds) = swap_within_escrow_for_direction<Coin1, LP1, Coin0, LP0>(
                swapper,
                swapper_address,
                market_address_1,
                market_address_0,
                emojicoin_1_balance,
                &mut escrow_ref_mut.emojicoin_1,
                &mut escrow_ref_mut.emojicoin_0
            );
        };

        (quote_volume, integrator_fee, emojicoin_0_proceeds, emojicoin_1_proceeds)
    }
}
