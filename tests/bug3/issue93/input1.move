module test {
    public entry fun test_invalid_offer_signer_capability(bob: signer) acquires Account {
        let (_alice_sk, alice_pk) = ed25519::generate_keys();
        let alice_pk_bytes = ed25519::validated_public_key_to_bytes(&alice_pk);
        let alice = create_account_from_ed25519_public_key(alice_pk_bytes);
        let alice_addr = signer::address_of(&alice);

        let bob_addr = signer::address_of(&bob);
        create_account(bob_addr);

        let challenge = SignerCapabilityOfferProofChallengeV2 {
            sequence_number: borrow_global<Account>(alice_addr).sequence_number,
            source_address: alice_addr,
            recipient_address: bob_addr
        };

        let sig = ed25519::sign_struct(&_alice_sk, challenge);

        // Maul the signature and make sure the call would fail
        let invalid_signature = ed25519::signature_to_bytes(&sig);
        let first_sig_byte = vector::borrow_mut(&mut invalid_signature, 0);
        *first_sig_byte += *first_sig_byte ^ 1;  // <-- here

        offer_signer_capability(
            &alice,
            invalid_signature,
            0,
            alice_pk_bytes,
            bob_addr
        );
    }
}