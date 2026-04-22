module Specs::Lemmas {
    spec fun sum(n: num): num {
        if (n == 0) { 0 }
        else {
            n + sum(n - 1)
        }
    }

    spec lemma monotonicity(x: num, y: num) {
        requires 0 <= x;
        requires x <= y;
        ensures sum(x) <= sum(y);
    } proof {
        if (x<y) {
            assert sum(y - 1) <= sum(y);
            apply monotonicity(x, y - 1);
        }
    }

    fun sum_up_to(n: u64): u64 {
        if (n == 0) { 0 }
        else {
            n + sum_up_to(n - 1)
        }
    }

    spec sum_up_to {
        aborts_if sum(n) > MAX_U64;
        ensures result == sum(n);
    } proof {
        forall x: num, y: num { sum(x), sum(y) }
        apply monotonicity(x, y);
    }

    spec module {
        lemma add_zero_left(x: u64) {
            ensures 0 + x == x;
        }

        lemma mul_comm(a: u64, b: u64) {
            ensures a *b == b *a;
        }
    }

    spec lemma strict_increase(a: u64, b: u64) {
        requires b == a + 1;
        ensures a<b;
    }

    struct Counter has drop {
        value: u64
    }

    fun increment(c: &mut Counter) {
        c.value = c.value + 1;
    }

    spec increment {
        requires c.value < MAX_U64;
        ensures c.value == old(c.value) + 1;
    } proof {
        assert c.value<MAX_U64;
        assert c.value + 1 <= MAX_U64;
    }

    fun add_and_return(c: &mut Counter, n: u64): u64 {
        c.value = c.value + n;
        c.value
    }

    spec add_and_return {
        requires c.value + n <= MAX_U64;
        ensures c.value == old(c.value) + n;
        ensures result == c.value;
    } proof {
        assert c.value + n <= MAX_U64;
        post assert c.value == old(c.value) + n;
        post assert result == c.value;
    }

    fun bump(c: &mut Counter) {
        c.value = c.value + 1;
    }

    spec bump {
        requires c.value < MAX_U64;
        ensures c.value == old(c.value) + 1;
        ensures old(c.value) < c.value;
    } proof {
        post apply strict_increase(old(c.value), c.value);
    }
}
