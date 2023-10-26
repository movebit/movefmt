module complex_module {  
  
    // Struct with nested comments and complex types  
    struct ComplexStruct2<T, U> {  
        // Field 1 comment  
        field1: /* Pre-comment */ vector<T> /* Inline comment */,  
        // Field 2 comment  
        field2: /* Comment before complex type */ SomeGenericStruct<U> /* Comment after complex type */,  
        // Field 3 comment  
        field3: /* Pre-comment */ optional<bool> /* Post-comment */,  
    } // Struct footer comment  
  
    // Function using the struct  
    fun use_complex_struct2(s: ComplexStruct2<u64, bool>) {  
        // Function comment  
    }  
}