module 0x1::test {
    entry fun swap<Coin0, LP0, Coin1, LP1>(swapper: &signer) acquires Escrow, Registry {
        // Crank schedule, set local variables.
        let (
            registry_ref_mut,
            melee_is_active,
            swapper_address,
            escrow_ref_mut,
            melee_id
        ) = existing_participant_prologue<Coin0, LP0, Coin1, LP1>(swapper);

        let (
            registry_ref_mut,
            melee_is_active,
            participant_address,
            escrow_ref_mut,
            melee_id
        ) = existing_participant_prologue<Coin0, LP0, Coin1, LP1>(participant);
    }
}
