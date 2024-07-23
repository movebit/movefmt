// separate_baseline: simplify
module 0x42::TestGlobalVars {
    use std::signer;
    use extensions::table::{Self, Table};

    // Counting
    fun sub(s: &signer) acquires T {
        spec {
            update sum_table = table::spec_set(
                sum_table,
                signer::address_of(s),
                table::spec_get(sum_table, signer::address_of(s)) - 1
            );
        };
    }
}
