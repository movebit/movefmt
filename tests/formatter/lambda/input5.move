module 0x42::LambdaTest1 {
    // This is a test case for comments in function parameters  
    public inline fun inline_mul(/* parameter a */ a: u64,   
                                 /* parameter b */ b: u64)   
        /* returns the product of a and b */ : u64 {  
        /* Multiply a and b */  
        a * b /* and return the result */  
    }  

    public inline fun inline_apply1(f: |u64|u64, b: u64) : u64 {
	inline_mul(f(b) + 1, inline_mul(3, 4))
    }

    public inline fun inline_apply(f: |u64|u64, b: u64) : u64 {
	f(b)
    }
}

module 0x42::LambdaTest2 {
    // Use statements with comments for each statement  
    use /* module */ 0x42::LambdaTest1;  
    use /* standard vector module */ std::vector;  
    public inline fun foreach<T>(v: &vector<T>, action: |&T|) { // expected to be not implemented
        let i = 0;
        while (i < vector::length(v)) {
            action(vector::borrow(v, i));
            i = i + 1;
        }
    }

    public fun test_inline_lambda() {
	let v = vector[1, 2, 3];
	let product = 1;
	foreach(&v, |e| product = LambdaTest1::inline_mul(product, *e));
    }

    public inline fun inline_apply2(g: |u64|u64, c: u64) : u64 {
	LambdaTest1::inline_apply1(|z|z, g(LambdaTest1::inline_mul(c, LambdaTest1::inline_apply(|x|x, 3)))) + 2
    }

    // Public inline function with comments for each parameter and the return value  
    public inline fun inline_apply3(/* lambda function g */ g: |u64|u64, /* value c */ c: u64) /* returns u64 */ : u64 {  
        // Apply the lambda function g to the result of applying another lambda function to c, multiply the result by 3, and add 4 to the result  
        LambdaTest1::inline_apply1(g, LambdaTest1::inline_mul(c, LambdaTest1::inline_apply(|x| { LambdaTest1::inline_apply(|y|y, x) }, 3))) + 4  
    }  
}

module 0x42::LambdaTest {
    use 0x42::LambdaTest2;

    public inline fun inline_apply(f: |u64|u64, b: u64) : u64 {
	f(b)
    }

    public inline fun inline_apply_test() : u64 {
	LambdaTest2::inline_apply2(|x| x + 1, 3) +
	LambdaTest2::inline_apply2(|x| x * x, inline_apply(|y|y, 3))
    }
}
