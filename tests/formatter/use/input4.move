module other_mod {
    struct SomeOtherStruct1 has drop {
        some_field: u64,
    }

    struct SomeOtherStruct2 has drop {
        some_field: u64,
    }

    struct SomeOtherStruct3 has drop {
        some_field: u64,
    }

    struct SomeOtherStruct4 has drop {
        some_field: u64,
    }

    struct SomeOtherStruct5 has drop {
        some_field: u64,
    }

    struct SomeOtherStruct6 has drop {
        some_field: u64,
    }

    struct SomeOtherStruct7 has drop {
        some_field: u64,
    }

    struct SomeOtherStruct8 has drop {
        some_field: u64,
    }

    struct SomeOtherStruct9 has drop {
        some_field: u64,
    }

    struct SomeOtherStruct10 has drop {
        some_field: u64,
    }

    struct SomeOtherStruct11 has drop {
        some_field: u64,
    }
    public fun some_other_struct(v: u64): SomeOtherStruct1 {
        SomeOtherStruct1 { some_field: v }
    }
}

module test_use2 {
    use module other_mod::{SomeOtherStruct1,SomeOtherStruct2,SomeOtherStruct3,SomeOtherStruct4,SomeOtherStruct5,SomeOtherStruct6,SomeOtherStruct7,SomeOtherStruct8,SomeOtherStruct9,SomeOtherStruct10,SomeOtherStruct11};
    use module other_mod::{SomeOtherStruct1, some_other_struct};
}