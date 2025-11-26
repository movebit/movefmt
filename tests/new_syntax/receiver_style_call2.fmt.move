// test_link_call
//# publish
module 0x42::m {
    struct S has drop {
        x: u64
    }

    fun plus_one(self: &S): S {
        self.x = self.x + 1;
        S { x: self.x }
    }

    fun test_link_call(s: S) {
        let p1m = &mut s;
        let p2m =
            p1m.plus_one()
                .plus_one()
                .plus_one()
                .plus_one()
                .plus_one()
                .plus_one()
                .plus_one()
                .plus_one()
                .plus_one()
                .plus_one()
                .plus_one()
                .plus_one();
    }
}

//# run 0x42::m::test
