module std::bit_vector {
    spec shift_left_for_verification_only {
        aborts_if false;
        ensures amount >= bitvector.length ==>
            (forall k in 0..bitvector.length: !bitvector.bit_field[k]);
        ensures amount < bitvector.length ==>
            (forall i in bitvector.length - amount..bitvector.length:
                !bitvector.bit_field[i]);
        ensures amount < bitvector.length ==>
            (
                forall i in 0..bitvector.length - amount:
                    bitvector.bit_field[i] == old(bitvector).bit_field[i + amount]
            );
    }

    spec schema WithdrawOnlyFromCapAddress<Token> {
        cap: WithdrawCapability;
        ensures forall addr: address where old(exists<Balance<Token>>(addr))
            && addr != cap.account_address:
            balance<Token>(addr) == old(balance<Token>(addr));
        ensures forall k in 0..bitvector.length:
            balance<Token>(addr) == old(balance<Token>(addr));

        axiom<CapState> forall c1: address, c2: address:
            (c1 == c2 ==>
                cap<CapState>(c1) == cap<CapState>(c2));
    }
}
