module test {
        public fun transfer<CoinType1, CoinType2>(
        from: &signer, to: address, amount: u64
    ) {
        BasicCoin::transfer<PoolToken<CoinType1, CoinType2>>(
            from,
            to,
            amount,
            PoolToken<CoinType1,
            CoinType2> {}
        );
    }

}