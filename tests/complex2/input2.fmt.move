// Script hash: 034204de
// Framework commit hash: 7bc3f195928488963bb947dc0e300f26527f2675
// Builder commit hash: 7bc3f195928488963bb947dc0e300f26527f2675
// Upgrade proposal for package `AptosStdlib`

// source digest: 1D4ED5929F42A72E62C7E3DF754A7EBB336DC734C24C5DFCC72E00B54CE886A0
script {
    use std::vector;
    use aptos_framework::aptos_governance;
    use aptos_framework::code;

    fun main(core_resources: &signer) {
        let framework_signer =
            aptos_governance::get_signer_testnet_only(
                core_resources,
                @0000000000000000000000000000000000000000000000000000000000000001
            );
        let code = vector::empty();
        let chunk0 = vector[
            161u8, 28u8, 235u8, 11u8, 6u8, 0u8, 0u8, 0u8, 5u8, 1u8, 0u8, 2u8, 2u8, 2u8,
            8u8, 7u8, 10u8, 44u8, 8u8, 54u8, 32u8, 10u8, 86u8, 10u8, 0u8, 0u8, 0u8, 1u8,
            0u8, 0u8, 0u8, 2u8, 0u8, 0u8, 16u8, 97u8, 108u8, 103u8, 101u8, 98u8, 114u8,
            97u8, 95u8, 98u8, 108u8, 115u8, 49u8, 50u8, 51u8, 56u8, 49u8, 2u8, 70u8, 114u8,
            11u8, 70u8, 114u8, 70u8, 111u8, 114u8, 109u8, 97u8, 116u8, 76u8, 115u8, 98u8,
            11u8, 100u8, 117u8, 109u8, 109u8, 121u8, 95u8, 102u8, 105u8, 101u8, 108u8,
            100u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 1u8, 0u8, 2u8, 1u8, 3u8, 1u8, 1u8, 2u8, 1u8, 3u8, 1u8, 0u8
        ];
        vector::push_back(&mut code, chunk0);
        let chunk1 = vector[
            161u8, 28u8, 235u8, 11u8, 6u8, 0u8, 0u8, 0u8, 6u8, 1u8, 0u8, 2u8, 3u8, 2u8,
            11u8, 5u8, 13u8, 5u8, 7u8, 18u8, 30u8, 8u8, 48u8, 32u8, 12u8, 80u8, 8u8, 0u8,
            0u8, 0u8, 1u8, 0u8, 1u8, 1u8, 0u8, 0u8, 2u8, 1u8, 1u8, 0u8, 1u8, 6u8, 9u8, 0u8,
            0u8, 5u8, 100u8, 101u8, 98u8, 117u8, 103u8, 5u8, 112u8, 114u8, 105u8, 110u8,
            116u8, 17u8, 112u8, 114u8, 105u8, 110u8, 116u8, 95u8, 115u8, 116u8, 97u8, 99u8,
            107u8, 95u8, 116u8, 114u8, 97u8, 99u8, 101u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8, 0u8, 1u8, 2u8, 0u8, 1u8, 1u8,
            2u8, 0u8, 0u8
        ];
    }
}
