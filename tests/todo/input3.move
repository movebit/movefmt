module bcs_stream {
    public fun deserialize_u256(stream: &mut BCSStream): u256 {
        let res =
            (*vector::borrow(data, cur) as u256)
                |((*vector::borrow(data, cur + 28) as u256) << 224)
                |((*vector::borrow(data, cur + 29) as u256) << 232) |(
                (*vector::borrow(data, cur + 30) as u256) << 240
            ) |((*vector::borrow(data, cur + 31) as u256) << 248);
    }
}
