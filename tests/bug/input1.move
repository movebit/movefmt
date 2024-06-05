// first module
module 0xc0ffee::m {
    fun test_return_with_nest() {
        return   (1, 2);
    }

    spec on_new_epoch(framework: &signer) {
        requires @std == signer::address_of(framework);
    }
}