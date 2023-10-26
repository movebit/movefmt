spec aptos_std::big_vector {

    // ---------------------
    // Spec helper functions
    // ---------------------

        spec fun spec_table_len<K, V>(t: TableWithLength<K, V>): u64 {
            table_with_length::spec_len(t)
        }

        spec fun spec_table_contains<K, V>(t: TableWithLength<K, V>, k: K): bool {
            table_with_length::spec_contains(t, k)
        }

    spec fun spec_at<T>(v: BigVector<T>/*comment*/, i: u64): T {
        let bucket = i / v.bucket_size;//comment
        //comment
        let idx =/*comment*/ i % v.bucket_size;
        /// comment
        let v = table_with_length::spec_get(v.buckets, /*comment*/bucket);
        /*comment*/
        v[idx]
    }
}
