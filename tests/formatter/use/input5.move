module other_mod1 {
    module other_mod2 {
        module other_mod3 {
            module other_mod4 {
                struct SomeOtherStruct4 has drop {
                    some_field: u64,
                }
                module other_mod5 {
                    struct SomeOtherStruct5 has drop {
                        some_field: u64,
                    }
                }
            }
        }
    }
}

module test_use2 {
    use module other_mod1::{other_mod2::{other_mod3::{other_mod4::{other_mod5::{SomeOtherStruct5}, SomeOtherStruct4}}}};
}