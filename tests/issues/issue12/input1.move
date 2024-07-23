module test {
    use std::signer; 

    fun test() : bool {
        signer::address_of(sign) == admin_address() || signer::address_of(sign) == @aptos_names || signer::address_of(sign) == @router_signer
    }
}