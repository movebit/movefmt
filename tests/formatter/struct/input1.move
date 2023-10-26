module test_module1 {  
  
    struct TestStruct1 {  
        // This is field1 comment  
        field1: u64,  
        field2: bool,  
    }  
}

module test_module2 {  
  
    struct TestStruct2 {  
        field1: u64, // This is a comment for field1  
        field2: bool, // This is a comment for field2  
    }  
}

module test_module3 {  
  
    // This is a comment before struct definition  
    struct TestStruct3 {  
        field1: u64,  
        field2: bool,  
    } // This is a comment after struct definition  
}

module test_module4 {  
  
    struct TestStruct4<T> {  
        // This is a comment before complex field  
        field: vector<T>, // This is a comment after complex field  
    }  
}

module test_module5 {  
  
    struct TestStruct5 {  
        field1: /* This is an inline comment */ u64,  
        field2: bool,  
    }  
}