module 0x42::LambdaTest1 {
    // Public inline function
    public inline fun inline_mul(
        a: u64, // Input parameter a
        b: u64
    ) // Input parameter b
        : u64 { // Returns a u64 value
        // Multiply a and b
        a * b
    }
}

module 0x42::LambdaTest2 {
    // Use statements
    use 0x42::LambdaTest1;
    use std::vector;

    // Public function with a comment before the function body
    public fun test_inline_lambda() {
        // Create a vector with elements 1, 2, and 3
        let v = vector[1, 2, 3];
        // Initialize a variable product to 1
        let product = 1;
        // Apply a lambda function to each element of the vector and update the product variable
        foreach(&v, |e| product = LambdaTest1::inline_mul(product, *e));
    }
}
