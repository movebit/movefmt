module fmt {
    use std::signer;

    struct Wrapper<T> has key, drop {
        fv: |T| u64 has copy + store + drop
    }

    #[persistent]
    fun test(f: || u64 has store + drop): u64 {
        if (f() == 1) 1 else 2
    }
    // store a function value of type `|T|u64` with `T = ||u64 has store+drop`
    public fun init(acc: &signer) {
        let f: | || u64 has store + drop | u64 has copy + store + drop = |x| test(x);
        move_to(acc, Wrapper { fv: f });
    }
}
