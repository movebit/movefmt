module test {
    fun t0(): R {
        (S { f: false }: S);

        let s = S { f: 0 };
        let r = (
            R {
                s: S { f: 0 },
                n2: Nat { f: s },
                n1: Nat { f: 0 },
                f: 0
            }: R
        );
        (
            R {
                s: r,
                f: false,
                n1: Nat { f: false },
                n2: Nat { f: r }
            }: R
        )
    }
}
