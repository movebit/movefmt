module std::bit_vector {
    use std::vector;

    /// Returns the length of the longest sequence of set bits starting at (and
    /// including) `start_index` in the `bitvector`. If there is no such
    /// sequence, then `0` is returned.
    public fun longest_set_sequence_starting_at(
        bitvector: &BitVector, start_index: u64
    ): u64 {
        assert!(start_index < bitvector.length, EINDEX);
        let index = start_index;

        // Find the greatest index in the vector such that all indices less than it are set.
        while ({
            spec {
                invariant index == start_index
                    || index - 1 < vector::length(bitvector.bit_field);
                invariant forall j in start_index..index:
                    j < vector::length(bitvector.bit_field);
            };
            index < bitvector.length
        }) {
            if (!is_index_set(bitvector, index)) break;
            index = index + 1;
        };

        index - start_index
    }
}
