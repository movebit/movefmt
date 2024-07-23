module test {
    public entry fun test_invalid_check_signer_capability_and_create_authorized_signer(
        bob: signer, charlie: signer
    ) acquires Account {
        let (alice_sk, alice_pk) = ed25519::generate_keys();
        let alice_pk_bytes = ed25519::validated_public_key_to_bytes(&alice_pk);
        let alice = create_account_from_ed25519_public_key(alice_pk_bytes);
        let alice_addr = signer::address_of(&alice);

        let bob_addr = signer::address_of(&bob);
        create_account(bob_addr);

        let challenge =
            SignerCapabilityOfferProofChallengeV2 {
                sequence_number: borrow_global<Account>(alice_addr).sequence_number,
                source_address: alice_addr,
                recipient_address: bob_addr,
            };

        let alice_signer_capability_offer_sig = ed25519::sign_struct(&alice_sk, challenge);

        offer_signer_capability(&alice,
            ed25519::signature_to_bytes(&alice_signer_capability_offer_sig),
            0,
            alice_pk_bytes,
            bob_addr);

        let alice_account_resource = borrow_global_mut<Account>(alice_addr);
        assert!(
            option::contains(&alice_account_resource.signer_capability_offer.for, &bob_addr),
                0);
    }

        #[verify_only]
    public fun shift_left_for_verification_only(bitvector: &mut BitVector, amount: u64) {
        if (amount >= bitvector.length) {
            let len = vector::length(&bitvector.bit_field);
            let i = 0;
            while ({
                spec {
                    invariant len == bitvector.length;
                    invariant forall k in 0..i: !bitvector.bit_field[k];
                    invariant forall k in i..bitvector.length: bitvector.bit_field[k] == old(bitvector).bit_field[k];
                };
                i < len
            }) {
                let elem = vector::borrow_mut(&mut bitvector.bit_field, i);
                *elem = false;
                i = i + 1;
            };
        } else {
            let i = amount;

            while ({
                spec {
                    invariant i >= amount;
                    invariant bitvector.length == old(bitvector).length;
                    invariant forall j in amount..i: old(bitvector).bit_field[j] == bitvector.bit_field[j - amount];
                    invariant forall j in (i-amount)..bitvector.length : old(bitvector).bit_field[j] == bitvector.bit_field[j];
                    invariant forall k in 0..i-amount: bitvector.bit_field[k] == old(bitvector).bit_field[k + amount];
                };
                i < bitvector.length
            }) {
                if (is_index_set(bitvector, i)) set(bitvector, i - amount)
                else unset(bitvector, i - amount);
                i = i + 1;
            };


            i = bitvector.length - amount;

            while ({
                spec {
                    invariant forall j in bitvector.length - amount..i: !bitvector.bit_field[j];
                    invariant forall k in 0..bitvector.length - amount: bitvector.bit_field[k] == old(bitvector).bit_field[k + amount];
                    invariant i >= bitvector.length - amount;
                };
                i < bitvector.length
            }) {
                unset(bitvector, i);
                i = i + 1;
            }
        }
    }


        spec rotate_authentication_key_with_rotation_capability(
        delegate_signer: &signer,
        rotation_cap_offerer_address: address,
        new_scheme: u8,
        new_public_key_bytes: vector<u8>,
        cap_update_table: vector<u8>
    ) {
        aborts_if !exists<Account>(rotation_cap_offerer_address);
        let delegate_address = signer::address_of(delegate_signer);
        let offerer_account_resource = global<Account>(rotation_cap_offerer_address);
        aborts_if !from_bcs::deserializable<address>(offerer_account_resource.authentication_key);
        let curr_auth_key = from_bcs::deserialize<address>(offerer_account_resource.authentication_key);
        aborts_if !exists<Account>(delegate_address);
        let challenge = RotationProofChallenge {
            sequence_number: global<Account>(delegate_address).sequence_number,
            originator: rotation_cap_offerer_address,
            current_auth_key: curr_auth_key,
            new_public_key: new_public_key_bytes,
        };
        /// [high-level-req-6.2]
        aborts_if !option::spec_contains(offerer_account_resource.rotation_capability_offer.for, delegate_address);
        /// [high-level-req-9.1]
        include AssertValidRotationProofSignatureAndGetAuthKeyAbortsIf {
            scheme: new_scheme,
            public_key_bytes: new_public_key_bytes,
            signature: cap_update_table,
            challenge,
        };

        let new_auth_key_vector = spec_assert_valid_rotation_proof_signature_and_get_auth_key(new_scheme, new_public_key_bytes, cap_update_table, challenge);
        let address_map = global<OriginatingAddress>(@aptos_framework).address_map;

        // Verify all properties in update_auth_key_and_originating_address_table
        aborts_if !exists<OriginatingAddress>(@aptos_framework);
        aborts_if !from_bcs::deserializable<address>(offerer_account_resource.authentication_key);
        aborts_if table::spec_contains(address_map, curr_auth_key) &&
            table::spec_get(address_map, curr_auth_key) != rotation_cap_offerer_address;

        aborts_if !from_bcs::deserializable<address>(new_auth_key_vector);
        let new_auth_key = from_bcs::deserialize<address>(new_auth_key_vector);

        aborts_if curr_auth_key != new_auth_key && table::spec_contains(address_map, new_auth_key);
        include UpdateAuthKeyAndOriginatingAddressTableAbortsIf {
            originating_addr: rotation_cap_offerer_address,
            account_resource: offerer_account_resource,
        };

        let post auth_key = global<Account>(rotation_cap_offerer_address).authentication_key;
        ensures auth_key == new_auth_key_vector;
    }
}