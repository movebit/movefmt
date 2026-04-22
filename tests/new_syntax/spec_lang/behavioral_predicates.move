module Specs::Behavioral {
    fun apply(f: |u64| u64, x: u64): u64 {
        f(x)
    }
    spec apply {
        ensures ensures_of<f>(x, result);
    }

    fun apply_may_abort(f: |u64| u64, x: u64): u64 {
        f(x)
    }
    spec apply_may_abort {
        aborts_if aborts_of<f>(x);
        ensures ensures_of<f>(x, result);
    }

    fun apply_no_abort(f: |u64| u64, x: u64): u64 {
        f(x)
    }
    spec apply_no_abort {
        requires !aborts_of<f>(x);
        aborts_if false;
        ensures ensures_of<f>(x, result);
    }

    fun apply_seq(f: |u64| u64 has copy, x: u64): u64 {
        f(f(x))
    }
    spec apply_seq {
        let y = result_of<f>(x);
        requires requires_of<f>(x) && requires_of<f>(y);
        aborts_if aborts_of<f>(x) || aborts_of<f>(y);
        ensures result == result_of<f>(y);
    }

    fun double(x: u64): u64 { x * 2 }
    spec double { ensures result == x * 2; }

    fun test_known(): u64 { double(5) }
    spec test_known {
        ensures result == result_of<double>(5);
    }
}