spec aptos_token::property_map {
    /// Abort according to the code
    spec create_property_value<T: copy>(data: &T): PropertyValue {
        use aptos_std::type_info::{type_name};

        let name = type_name<T>();
        aborts_if !string::spec_internal_check_utf8(b"bool");

        aborts_if name != spec_utf8(b"bool")
            && !string::spec_internal_check_utf8(b"u8");

        aborts_if name != spec_utf8(b"bool")
            && name != spec_utf8(b"u8")
            && !string::spec_internal_check_utf8(b"u64");

        aborts_if name != spec_utf8(b"bool")
            && name != spec_utf8(b"u8")
            && name != spec_utf8(b"u64")
            && !string::spec_internal_check_utf8(b"u128");

        aborts_if name != spec_utf8(b"bool")
            && name != spec_utf8(b"u8")
            && name != spec_utf8(b"u64")
            && name != spec_utf8(b"u128")
            && !string::spec_internal_check_utf8(b"address");

        aborts_if name != spec_utf8(b"bool")
            && name != spec_utf8(b"u8")
            && name != spec_utf8(b"u64")
            && name != spec_utf8(b"u128")
            && name != spec_utf8(b"address")
            && !string::spec_internal_check_utf8(b"0x1::string::String");

        aborts_if name != spec_utf8(b"bool")
            && name != spec_utf8(b"u8")
            && name != spec_utf8(b"u64")
            && name != spec_utf8(b"u128")
            && name != spec_utf8(b"address")
            && name != spec_utf8(b"0x1::string::String")
            && !string::spec_internal_check_utf8(b"vector<u8>");
    }
}
