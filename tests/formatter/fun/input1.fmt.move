module TestFunFormat {

    struct SomeOtherStruct has drop {
        some_field: u64
    }

    public fun test_long_fun_name_lllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllll(
        v: u64
    ): SomeOtherStruct {
        SomeOtherStruct { some_field: v }
    }

    // test fun sign whith no whitespace
    public fun multi_arg(p1: u64, p2: u64): u64 {
        p1 + p2
    }
}
