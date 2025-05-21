use crate::item_enum::define_config_type_on_enum;
use proc_macro2::TokenStream;

/// Defines `config_type` on enum.
pub fn define_config_type(input: &syn::Item) -> TokenStream {
    match input {
        syn::Item::Enum(en) => define_config_type_on_enum(en),
        _ => panic!("Expected enum or struct"),
    }
    .unwrap()
}
