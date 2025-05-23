address 0x42 {
module example {
    // all 3 of these functions are equivalent

    // when no return type is provided, it is assumed to be `()`
    fun returs_unit_1() {}

    // there is an implicit () value in empty expression blocks
    fun returs_unit_2(): () {}

    // explicit version of `returs_unit_1` and `returs_unit_2`
    fun returs_unit_3(): () {
        ()
    }

    fun returns_3_values(): (u64, bool, address) {
        // comment1
        (0, false, @0x42) // comment2
    }
}
}
