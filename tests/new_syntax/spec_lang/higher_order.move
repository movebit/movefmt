module Specs::HigherOrder {
    struct Counter has key, store { value: u64 }

    fun contains(v: &vector<u64>, pred: |&u64| bool has copy + drop): bool {
        let i = 0;
        let len = std::vector::length(v);
        while (i < len) {
            if (pred(std::vector::borrow(v, i))) {
                return true;
            };
            i = i + 1;
        }
        spec {
            invariant i <= len;
            invariant forall j in 0..i: !result_of<pred>(v[j]);
        };
        false
    }
    spec contains {
        requires forall x in 0..len(v): !aborts_of<pred>(v[x]);
        aborts_if false;
        ensures result == (exists k in 0..len(v): result_of<pred>(v[k]));
    }

    spec fun spec_reduce(reducer: |u64, u64|u64, v: vector<u64>, val: u64, end: u64): u64 {
        if (end == 0) val
        else {
            let val = spec_reduce(reducer, v, val, end - 1);
            result_of<reducer>(val, v[end - 1])
        }
    }

    fun reduce(vec: vector<u64>, start: u64, reducer: |u64, u64|u64 has copy + drop): u64 {
        let i = 0;
        let len = std::vector::length(vec);
        let acc = start;
        while (i < len) {
            acc = reducer(acc, *std::vector::borrow(vec, i));
            i = i + 1;
        };
        spec {
            invariant i <= len;
            invariant acc == spec_reduce(reducer, vec, start, i);
        };
        acc
    }
    spec reduce {
        ensures result == spec_reduce(reducer, vec, start, len(vec));
    }

    fun apply_void_mut(f: |&mut u64|, x: &mut u64) { f(x) }
    spec apply_void_mut {
        ensures x == result_of<f>(old(x));
    }

    fun apply_mut(f: |&mut u64| u64, x: &mut u64): u64 { f(x) }
    spec apply_mut {
        ensures ensures_of<f>(old(x), result, x);
    }

    fun apply_mut_result(f: |&mut u64| u64, x: &mut u64): u64 { f(x) }
    spec apply_mut_result {
        ensures (result, x) == result_of<f>(old(x));
    }

    spec apply_mut_extract {
        ensures result == {let (r, _p) = result_of<f>(old(x)); r};
        ensures x == {let (_r, p) = result_of<f>(old(x)); p};
    }
}