// first module
module 0xc0ffee::m {
    fun test_return_with_nest() {
        let one_aptos_coin = 10 **(coin::decimals<Aptoscoin>() as u64);
        return   (1, 2);
    }

    spec on_new_epoch(framework: &signer) {
        requires @std == signer::address_of(framework);
    }
}