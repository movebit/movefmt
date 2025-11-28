module 0x42::M {
    fun fmt_error(address: address, x: u64, address2: address): () {

        let taker1_expected_fill_sizes = vector::empty<u64>();
        let taker1_total_fill_size = taker1_expected_fill_sizes.fold(0, |acc, x| acc + x);
    }
}
