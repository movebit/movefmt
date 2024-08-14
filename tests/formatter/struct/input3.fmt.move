module complex_module {

    // Struct with comments in various positions
    struct ComplexStruct1<T, U> {
        // Field 1 comment
        field1: vector<U>, // Trailing comment for field1
        // Field 2 comment
        field2: bool,
        // Field 3 comment
        field3: /* Pre-comment */ SomeOtherStruct<T> /* Post-comment */
    } /* Struct footer comment */

    // Struct with nested comments and complex types
    struct ComplexStruct2<T, U> {
        // Field 1 comment
        field1: /* Pre-comment */ vector<T> /* Inline comment */,
        // Field 2 comment
        field2: /* Comment before complex type */ SomeGenericStruct<U> /* Comment after complex type */,
        // Field 3 comment
        field3: /* Pre-comment */ optional<bool> /* Post-comment */
    } // Struct footer comment
}
