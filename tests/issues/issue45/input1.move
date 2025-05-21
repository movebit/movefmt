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
        let nummmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm: u64 = 0;
        let llllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllll = 1;
        nummmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm = llllllllllllllllllllllllllllllllllllllllllllllllllllllllllllllll;
        nummmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm -= 1;
        nummmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm += 2;
        nummmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm *= 3;
        nummmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm /= 4;
        nummmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm %= 5;
        nummmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm ^= 6;
        nummmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm |= 7;
        nummmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm &= 8;
        nummmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm
    }
}
