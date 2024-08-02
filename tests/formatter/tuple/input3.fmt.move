address 0x42 {
module example {
    // there is an implicit () value in empty expression blocks
    fun returs_unit_2(): () { /*comment*/ }

    // explicit version of `returs_unit_1` and `returs_unit_2`
    fun returs_unit_3(): () {
        ( /*comment*/ )
    }

    fun returns_3_values(): (u64, bool, address) {
        // comment1
        (
            // comment2
            0,
            /*comment3*/ false /*comment4*/, @0x42
        ) // comment5
    }
}
}
