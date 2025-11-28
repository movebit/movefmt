#[fmt::skip]
module test_branch1 {
        /// Return the expected bucket index to find the hash.
    /// Basically, it use different base `1 << level` vs `1 << (level + 1)` in modulo operation based on the target
    /// bucket index compared to the index of the next bucket to split.
    fun bucket_index(level: u8, num_buckets: u64, hash: u64): u64 {
        let index = hash % (1 << (level + 1));
            if (index < num_buckets) {
            // in existing bucket
            index } else {
            // in unsplitted bucket
            index % (1 << level)
        }
    }
}

module test_branch2 {
    fun bucket_index(level: u8, num_buckets: u64, hash: u64): u64 {
        let index = hash % (1 << (level + 1));
        if (index < num_buckets) { index }
        else {
            index % (1 << level)
        }
    }
}

#[fmt::skip]
module test_branch3 {
    fun bucket_index(level: u8, num_buckets: u64, hash: u64): u64 {
        let index = hash % (1 << (level + 1));
            if (index < num_buckets) {
                /*commet*/
            index } else {
            index % (1 << level)
        }
    }
}

module test_branch4 {
    fun bucket_index(level: u8, num_buckets: u64, hash: u64): u64 {
        let index = hash % (1 << (level + 1));
        if (index < num_buckets) { /*commet*/
            index
        } else {
            index % (1 << level)
        }
    }
}

#[fmt::skip]
module test_branch5 {
    fun bucket_index(level: u8, num_buckets: u64, hash: u64): u64 {
        let index = hash % (1 << (level + 1));
        if (index < num_buckets) {/*commet*/index } else {
            index % (1 << level)
        }
    }
}
