module 0x42::LambdaTest1 {
    /** Public inline function */
    public inline fun inline_mul(
        /** Input parameter a */ a: u64,
        /** Input parameter b */ b: u64
    )
        /** Returns a u64 value */ : u64 {
        /** Multiply a and b */
        a * b
    }

    public inline fun inline_apply1(f: |u64| u64, b: u64): u64 {
        inline_mul(f(b) + 1, inline_mul(3, 4))
    }

    public inline fun inline_apply(f: |u64| u64, b: u64): u64 {
        f(b)
    }
}

module 0x42::LambdaTest2 {
    /** Use statements */
    use 0x42::LambdaTest1;
    use std::vector;

    // Public inline function with comments for parameters and return value
    public inline fun inline_apply2(
        /* function g */ g: |u64| u64, /* value c */ c: u64
    ) /* returns u64 */ : u64 {
        // Apply the lambda function g to the result of applying another lambda function to c, and add 2 to the result
        LambdaTest1::inline_apply1(
            |z| z,
            g(
                LambdaTest1::inline_mul(c, LambdaTest1::inline_apply(|x| x, 3))
            )
        ) + 2
    }
}
