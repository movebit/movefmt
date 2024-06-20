module test_big_pragmas {
    #[test(
        router = @router,
        aptos_names = @aptos_names,
        aptos_names_v2_1 = @aptos_names_v2_1,
        user1 = @0x077,
        user2 = @0x266f,
        aptos = @0x1,
        foundation = @0xf01d
    )]
    fun test() {}

}
