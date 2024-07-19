module 0x42::M {
    struct S {
        f: u64
    }

    fun t1(u: &u64): u64 {
        if (true) return*u;
        0
    }

    fun t2(s: &S): &u64 {
        if (true) return&s.f else&s.f
    }
}