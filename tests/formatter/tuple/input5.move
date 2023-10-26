address 0x42 {  
    module example {  
        /**  
         * This function doesn't return anything. It's just for demonstration purposes.  
         */  
        fun returns_unit() {/*comment*/}  
          
        /**  
         * This function returns a tuple containing two boolean values.  
         */  
        fun returns_2_values(): /*(bool, bool)*/ (bool, bool) { (true, false) }  
          
        /**  
         * This function returns a tuple containing four values: a reference to a u64 value,  
         * a u8 value, a u128 value, and a vector of u8 values.  
         */  
        fun returns_4_values(x: &u64): /*(&u64, u8, u128, vector<u8>)*/ (&u64, u8, u128, vector<u8>) { (x, /*comment*/0, 1, b"foobar") }  
          
        /**  
         * This function demonstrates various examples using tuples.  
         * It includes assignments to tuple variables and reassignments using conditional statements.  
         */  
        fun examples(cond: bool) {  
            // Assignment of unit value to a variable  
            let () = ();  
              
            // Assignment of tuple values to variables x and y  
            let (x, y): /*(u8, u64)*/ (u8, u64) = (0, /*comment*/1);  
              
            // Assignment of tuple with multiple types to variables a, b, c, and d  
            let (a, b, c, d) = /*(@0x0, 0, false, b"")*/ (@0x0, 0, false, b"");  
              
            // Reassignment of unit value to a variable  
            () = ();  
              
            // Conditional reassignment of tuple values x and y  
            (x, y) = if (cond) /*(1, 2)*/ (1, 2) else /*(3, 4)*/ (3, 4);  
              
            // Reassignment of tuple values a, b, c, and d  
            (a, b, c, d) = /*(@0x1, 1, true, b"1")*/ (@0x1, 1, /*comment*/true, b"1");  
        }  
          
        /**  
         * This function demonstrates examples using function calls that return tuples.  
         */  
        fun examples_with_function_calls() {  
            // Calling a function that returns unit and assigning the result to a variable  
            let () = returns_unit();  
              
            // Calling a function that returns a tuple of booleans and assigning the result to variables x and y  
            let (x, y): /*(bool, bool)*/ (bool /*comment*/, bool) = returns_2_values();  
              
            // Calling a function that returns a tuple of multiple types and assigning the result to variables a, b, c, and d  
            let (a, b, c, d) = returns_4_values(&0);  
              
            // Reassignment using function call that returns unit  
            () = returns_unit();  
              
            // Reassignment using function call that returns a tuple of booleans  
            (x, y) = returns_2_values();  
              
            // Reassignment using function call that returns a tuple of multiple types  
            (a, b, c, d) = returns_4_values(&1);  
        }  
    }  
}