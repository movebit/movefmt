module test {
    #[test]
    fun test_for_each() {
        let v = vector[1, 2, 3];
        let s = 0;
        V::for_each(v, |e| {
            s = s + e;
        });
        assert!(s == 6, 0)
    }

    public(friend) fun calculate_max_quote_match(
        direction: bool, taker_fee_divisor: u64, max_quote_delta_user: u64
    ): u64 {
        // Calculate numerator for both buy and sell equations.
        let numerator = (taker_fee_divisor as u128) * (max_quote_delta_user as u128);
        // Calculate denominator based on direction.
        let denominator =
            if (direction == BUY) (taker_fee_divisor + 1 as u128)
            else (taker_fee_divisor - 1 as u128);
        // Calculate maximum quote coins to match.
        let max_quote_match = numerator / denominator;
        // Return corrected sell overflow match amount if needed,
        if (max_quote_match > (HI_64 as u128)) HI_64
        else (max_quote_match as u64) // Else max quote match amount.
    }
}
