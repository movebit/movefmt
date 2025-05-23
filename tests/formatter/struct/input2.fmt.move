module complex_module {

    // Module comment
    use std::optional;

    // Struct with comments in various positions
    struct ComplexStruct1<T, U> {
        // Field 1 comment
        field1: /* Inline comment */ optional<T>,
        // Field 2 comment
        field2: vector<U>, // Trailing comment for field2
        // Field 3 comment
        field3: bool,
        // Field 4 comment
        field4: /* Pre-comment */ SomeOtherStruct<T> /* Post-comment */
    } /* Struct footer comment */

    // Function using the struct
    fun use_complex_struct1(s: ComplexStruct1<u64, bool>) {
        // Function comment
    }
}
