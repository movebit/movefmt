module test_assign_with_binop1 {
    fun bucket_index(num: u64): u64 {
        num = 1;
        num -= 1;
        num += 1;
        num *= 1;
        num /= 1;
        num %= 1;
        num ^= 1;
        num |= 1;
        num &= 1;
    }
}

module test_assign_with_binop1 {
    fun bucket_index(num: u64): u64 {
        let llllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllll = 1;
        num = llllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllll;
        num -= llllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllll;
        num += llllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllll;
        num *= llllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllll;
        num /= llllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllll;
        num %= llllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllll;
        num ^= llllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllll;
        num |= llllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllll;
        num &= 1llllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllll;
    }
}
