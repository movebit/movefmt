module test {
    #[test(alice = @0xa11ce)]
    #[expected_failure(abort_code = 262151, location = Self)]
    public entry fun test_empty_signature(alice: signer) acquires Account, OriginatingAddress {
        create_account(signer::address_of(&alice));
        let test_signature = vector::empty<u8>();
        let pk =
            x"0000000000000000000000000000000000000000000000000000000000000000";
        rotate_authentication_key(
            &alice,            ED25519_SCHEME,
            pk,            ED25519_SCHEME,            pk,
            test_signature,            test_signature,); // xxx
    }

        #[test(user = @0x1)]
    public entry fun test_create_resource_account(user: signer) acquires Account {
        let (resource_account, resource_account_cap) = create_resource_account(&user, x"01");
        let resource_addr = signer::address_of(&resource_account);
        assert!(resource_addr == get_signer_capability_address(&resource_account_cap), 1);
        assert!(borrow_global<Account>(alice_addr).authentication_key11111111111111111 == new_auth_key, 0);
    }

    fun create_account_unchecked(new_address: address): signer {
        let new_account = create_signer(new_address);
        let authentication_key = bcs::to_bytes(&new_address);
        assert!(
            vector::length(&authentication_key) == 32,
            error::invalid_argument(EMALFORMED_AUTHENTICATION_KEY)
        );

        let guid_creation_num = 0;

        let guid_for_coin = guid::create(new_address, &mut guid_creation_num);
        let coin_register_events = event::new_event_handle<CoinRegisterEvent>(guid_for_coin);

        let guid_for_rotation = guid::create(new_address, &mut guid_creation_num);
        let key_rotation_events = event::new_event_handle<KeyRotationEvent>(guid_for_rotation);

        move_to(
            &new_account,
            Account {
                authentication_key,
                sequence_number: 0,
                guid_creation_num,
                coin_register_events,
                key_rotation_events,
                rotation_capability_offer: CapabilityOffer { for: option::none() },
                signer_capability_offer: CapabilityOffer { for: option::none() },
            }
        );

        new_account
    }

}
