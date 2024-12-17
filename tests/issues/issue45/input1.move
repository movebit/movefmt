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

module test_assign_with_binop2 {
    fun bucket_index():u64 {
        let nummmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm: u64 = 0;
        let llllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllll = 1;
        nummmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm = llllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllll;
        nummmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm -= llllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllll;
        nummmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm += llllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllll;
        nummmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm *= llllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllll;
        nummmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm /= llllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllll;
        nummmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm %= llllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllll;
        nummmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm ^= llllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllll;
        nummmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm |= llllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllll;
        nummmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm &= llllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllll;
        nummmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm
    
    }
}
