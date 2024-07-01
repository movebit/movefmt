address 0x42 {
module example {
    /**
    * This function doesn't return anything. It's just for demonstration purposes. */
    fun returns_unit() { /*comment*/ }

    /**
    * This function returns a tuple containing two boolean values.
    */
    fun returns_2_values(): /*(bool, bool)*/ (bool, bool) {
        (true, false)
    }

    /**
    * This function returns a tuple containing four values: a reference to a u64 value,
    * a u8 value, a u128 value, and a vector of u8 values.
    */
    fun returns_4_values(x: &u64): /*(&u64, u8, u128, vector<u8>)*/ (&u64, u8, u128, vector<u8>) {
        (x, /*comment*/ 0, 1, b"foobar")
    }

    /**
    * This function demonstrates various examples using tuples.
    * It includes assignments to tuple variables and reassignments using conditional statements.
    */
    fun examples(cond: bool) {
        // Assignment of tuple values to variables x and y
        let (x, y): /*(u8, u64)*/ (u8, u64) = (0, /*comment*/ 1);
    }
}
}
