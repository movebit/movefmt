module test_module1 {

    struct TestStruct1 {
        // This is field1 comment
        field1: u64,
        field2: bool
    }
}

module test_module2 {

    struct TestStruct2 { // This is a comment before struct definition
        field1: u64, // This is a comment for field1
        field2: bool // This is a comment for field2
    } // This is a comment after struct definition
}

module test_module4 {

    struct TestStruct4<T> {
        // This is a comment before complex field
        field: vector<T> // This is a comment after complex field
    }
}
