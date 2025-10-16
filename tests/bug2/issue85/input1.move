module A {
    use 0x2::S;

    // I1: <concrete, concrete>
    invariant exists<S::Storage<u64, bool>>(@0x22) ==>
        global<S::Storage<u64, bool>>(@0x22).x == 1;

    // I4: <generic, generic>
    invariant<X, Y>(exists<S::Storage<X, Y>>(@0x25) && exists<S::Storage<X, Y>>(@0x26)) ==>

    global<S::Storage<X, Y>>(@0x25) == global<S::Storage<X, Y>>(@0x26);

    public fun good(account1: signer, account2: signer) {
        S::publish_x_y<u64, bool>(account1, 1, true);
        S::publish_x_y<u64, bool>(account2, 1, true);
    }
}