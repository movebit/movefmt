module Specs::ProofBlocks {
    fun double(x: u64): u64 {
        x + x
    }

    spec double {
        aborts_if 2 * x > 18446744073709551615;
        ensures result == 2 * x;
    } proof {
        assert x + x == 2 *x;
    }

    fun weighted_avg_x2(x: u64, y: u64): u64 {
        (3 * x + y) / 4 * 2
    }

    spec weighted_avg_x2 {
        requires 3 * x + y <= 18446744073709551615;
        ensures result == (3 * x + y) / 4 * 2;
        ensures result <= 3 * x + y;
    } proof {
        let wx = 3 *x;
        let sum = wx + y;
        let half = sum / 4;
        assert half <= sum;
        assert half * 2 <= sum;
    }

    fun div3_le(x: u64): u64 {
        x / 3
    }

    spec div3_le {
        ensures result <= x;
    } proof {
        assume[trusted] x / 3 <= x;
    }

    fun square_plus_one(x: u64): u64 {
        (x + 1) * (x + 1)
    }

    spec square_plus_one {
        requires x + 1 <= 4294967295;
        ensures result == (x + 1) * (x + 1);
    } proof {
        let y = x + 1;
        let r = y *y;
        assert r == (x + 1) *(x + 1);
        post assert r == result;
    }

    fun max(a: u64, b: u64): u64 {
        if (a >= b) { a }
        else { b }
    }

    spec max {
        ensures result >= a;
        ensures result >= b;
        ensures result == a || result == b;
    } proof {
        if (a >= b) {
            post assert result == a;
            assert a >= a;
            assert a >= b;
        } else {
            post assert result == b;
            assert b> a;
            assert b >= b;
        }
    }

    fun double_post(x: u64): u64 {
        x + x
    }

    spec double_post {
        requires x + x <= 18446744073709551615;
        ensures result == 2 * x;
    } proof {
        assert x + x == 2 *x;
        post assert result == x + x;
    }

    fun shift_add(x: u64, y: u64): u64 {
        x * 2 + y
    }

    spec shift_add {
        requires x * 2 + y <= 18446744073709551615;
        ensures result == x * 2 + y;
    } proof {
        let doubled = x * 2;
        assert doubled + y <= 18446744073709551615;
        post {
            let expected = doubled + y;
            assert result == expected;
        }
    }

    fun add_three(x: u64): u64 {
        x + 1 + 1 + 1
    }

    spec add_three {
        requires x + 3 <= 18446744073709551615;
        ensures result == x + 3;
    } proof {
        calc(x + 1 + 1 + 1 == x + 2 + 1 == x + 3);
    }

    fun double_plus_one(x: u64): u64 {
        2 * x + 1
    }

    spec double_plus_one {
        requires 2 * x + 1 <= 18446744073709551615;
        ensures result >= x;
    } proof {
        calc(2 *x + 1 >= 2 *x >= x);
    }

    fun abs_diff(a: u64, b: u64): u64 {
        if (a >= b) { a - b }
        else { b - a }
    }

    spec abs_diff {
        ensures result == if (a >= b) { a - b }
        else { b - a };
    } proof {
        split a >= b;
    }
}
