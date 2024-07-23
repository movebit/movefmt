module test_spec_fun_has_many_para {
        spec rotate_authentication_key_with_rotation_capability(
        delegate_signer: &signer,
        rotation_cap_offerer_address: address,
        new_scheme: u8,
        new_public_key_bytes: vector<u8>,
        cap_update_table: vector<u8>
    ) {
        aborts_if !exists<Account>(rotation_cap_offerer_address);
    }
}