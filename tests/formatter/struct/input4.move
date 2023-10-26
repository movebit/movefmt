module complex_module {  
  
    // Struct with various comment styles and positions  
    struct ComplexStruct3 {  
        // Field 1 comment (single-line)  
        field1: u64, // Inline comment (single-line)  
        /* Field 2 comment (multi-line) */  
        field2: /* Inline comment (multi-line) */ bool, // Trailing comment (single-line)  
    } // Struct footer comment (single-line)  
  
    // Function using the struct  
    fun use_complex_struct3(s: ComplexStruct3) {  
        // Function comment (single-line)  
    }  
}

module complex_module2 {  
  
    // Struct with comments in nested types and fields with expressions  
    struct ComplexStruct4<T> {  
        // Field 1 comment with expression and nested comments  
        field1: /* Pre-comment */ SomeStruct<T> /* Inline comment */ { field: /* Nested comment */ expression },  
        // Field 2 comment with expression and nested comments in a complex type  
        field2: /* Pre-comment */ SomeOtherGenericStruct<T> /* Inline comment */ { field1: expression1, field2: /* Nested comment */ expression2 },  
    } // Struct footer comment with expression and nested comments in a complex type, } // Inline comment, More footer comment lines...   
} // Struct footer comment (single-line) with trailing comments after it... // Trailing comment line 1 // Trailing comment line 2... // Trailing comment line N... } // Module footer comment with trailing comments after it... // Module trailing comment line 1 // Module trailing comment line 2... // Module trailing comment line M... </T> </T> </U> </T> </U> </T> </U> </T> </U> </T> </U> </T> </U> </T> </U> </T> </U> </T> </U> </T> </U> </T> </U> </T> </U> </T> </U> </T> </U> </T> </U> </T> </U> </T> </U> </T> </U> </T> </U> </T> </U> </T> </U> </T> </U> </T> </U></module>  // Module footer comment (single-line) with trailing comments after it...</module></module></module></module></module></module></module></module></module></module></module></module></module></module></module></module></module></module></module></module></module></module></module></module></module></module></module></module></module></module></module></module></module></module></module></module></module></module></module></module></module
