module bcs_stream {
    public fun deserialize_u256(stream: &mut BCSStream): u256 {
        let res =
            (*vector::borrow(data, cur) as u256)
                |((*vector::borrow(data, cur + 28) as u256) << 224)
                |((*vector::borrow(data, cur + 29) as u256) << 232) |(
                (*vector::borrow(data, cur + 30) as u256) << 240
            ) |((*vector::borrow(data, cur + 31) as u256) << 248);
    }

    inline fun get_hero(
        creator: &address, collection: &String, name: &String
    ): (Object<Hero>, &Hero) {
        let token_address = token::create_token_address(creator, collection, name);
        (
            object::address_to_object<Hero>(token_address), borrow_global<Hero>(
                token_address
            )
        )
    }

    spec foo {
        ensures result
            == (((((1 as u64) & (0 as u64)) as u64) | (((1 as u64) & (1 as u64)) as u64) as u64) | (
                ((1 as u64) ^(2 as u64)) as u64
            ) as u64);
    }
}
