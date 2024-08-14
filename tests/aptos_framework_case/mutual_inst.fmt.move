address 0x2 {
module S {
    struct Storage<X: store, Y: store> has key {
        x: X,
        y: Y,
        v: u8
    }

    // F1: <concrete, concrete>
    public fun publish_u64_bool(account: signer, x: u64, y: bool) {
        move_to(&account, Storage { x, y, v: 0 })
    }
}

module A {
}
}
