spec aptos_std::big_vector {
    // -----------------
    // Data invariants
    // -----------------

    spec BigVector {
        invariant bucket_size != 0;
        invariant spec_table_len(buckets) == 0 ==> end_index == 0;
        invariant end_index == 0 ==> spec_table_len(buckets) == 0;
        invariant end_index <= spec_table_len(buckets) * bucket_size;

        // ensure all buckets except last has `bucket_size`
        invariant spec_table_len(buckets) == 0
            || (forall i in 0..spec_table_len(buckets)-1: len(table_with_length::spec_get(buckets, i)) == bucket_size);
        // ensure last bucket doesn't have more than `bucket_size` elements
        invariant spec_table_len(buckets) == 0
            || len(table_with_length::spec_get(buckets, spec_table_len(buckets) -1 )) <= bucket_size;
        // ensure each table entry exists due to a bad spec in `Table::spec_get`
        invariant forall i in 0..spec_table_len(buckets): spec_table_contains(buckets, i);
        // ensure correct number of buckets
        invariant spec_table_len(buckets) == (end_index + bucket_size - 1) / bucket_size;
        // ensure bucket lengths add up to `end_index`
        invariant (spec_table_len(buckets) == 0 && end_index == 0)
            || (spec_table_len(buckets) != 0 && ((spec_table_len(buckets) - 1) * bucket_size) + (len(table_with_length::spec_get(buckets, spec_table_len(buckets) - 1))) == end_index);
        // ensures that no out-of-bound buckets exist
        invariant forall i: u64 where i >= spec_table_len(buckets):  {
            !spec_table_contains(buckets, i)
        };
        // ensures that all buckets exist
        invariant forall i: u64 where i < spec_table_len(buckets):  {
            spec_table_contains(buckets, i)
        };
        // ensures that the last bucket is non-empty
        invariant spec_table_len(buckets) == 0
            || (len(table_with_length::spec_get(buckets, spec_table_len(buckets) - 1)) > 0);
    }

    // -----------------------
    // Function specifications
    // -----------------------

spec empty<T: store>(bucket_size: u64): BigVector<T> {
    aborts_if bucket_size == 0;
    ensures length(result) == 0;
    ensures result.bucket_size == bucket_size;
}

spec singleton<T: store>(element: T, bucket_size: u64): BigVector<T> {
    ensures length(result) == 1;
    ensures result.bucket_size == bucket_size;
}

    // ---------------------
    // Spec helper functions
    // ---------------------

        spec fun spec_table_len<K, V>(t: TableWithLength<K, V>): u64 {
            table_with_length::spec_len(t)
        }

        spec fun spec_table_contains<K, V>(t: TableWithLength<K, V>, k: K): bool {
            table_with_length::spec_contains(t, k)
        }

    spec fun spec_at<T>(v: BigVector<T>, i: u64): T {
        let bucket = i / v.bucket_size;
        let idx = i % v.bucket_size;
        let v = table_with_length::spec_get(v.buckets, bucket);
        v[idx]
    }
}
