module just_one_item_per_line {
    module BlockComment {
        use aptos_std::type_info::{
            /* use_item before */Self, 
            TypeInfo
        };

        use econia::tablist::{
            Self, 
            /* use_item before */Tablist};
    }

    module InlineComment {
        use aptos_std::type_info::{
            // use_item
            Self, 
            TypeInfo
        };

        use econia::tablist::{
            Self, 
            // use_item
            Tablist};
    }
}