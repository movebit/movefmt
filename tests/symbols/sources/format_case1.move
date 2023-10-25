module Symbols::M1 {
    use aptos_framework::aptos_coin::AptosCoin;
        use aptos_framework::coin::{Self, Coin}/* use */;
        use aptos_std::type_info::{Self/* use_item */, TypeInfo};
            use econia::resource_account;
            use econia::tablist::{Self, Tablist/* use_item */};
        use std::signer::address_of;
    use std::vector;

    struct SomeStruct has key, drop, store {
            some_field/* some_field */: u64,
        }

        const SOME_CONST: u64 = 42;

        /// unpack
        fun unpack(s: SomeStruct): u64 {
            //xxx
            let SomeStruct { some_field: value } = s;
            value
        }

        fun cp(value /* test_para_name */: u64 /* test_para_note */): u64 {
            let ret = value; //xxx
            ret
        }/* note after fun body block */

        fun pack(): SomeStruct {
            let ret = SomeStruct { some_field: SOME_CONST };
            ret
        }

        fun other_mod_struct(): Symbols::/* module reference */M2::SomeOtherStruct/* note after fun sign */ {
            Symbols::M2::some_other_struct(SOME_CONST)
        }
}
