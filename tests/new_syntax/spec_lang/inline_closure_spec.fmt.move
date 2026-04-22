module Specs::InlineClosure {
    fun guarded_apply(x: u64): u64 {
        guarded_apply(
            |y| {
                if (y > 500) abort 1;
                y * 2
            }
            spec {
                aborts_if y > 500;
                ensures result == y * 2;
            },
            x
        )
    }

    fun test_opaque(x: u64): u64 {
        apply_opaque(|y| y + 5 spec {
            ensures result == y + 5;
        }, x)
    }

    fun apply_opaque(f: |u64| u64, x: u64): u64 {
        f(x)
    }

    spec apply_opaque {
        pragma opaque = true;
        ensures ensures_of<f> (x, result);
    }
}
