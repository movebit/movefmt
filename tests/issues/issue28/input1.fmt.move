module test {
    #[callable]
    fun f1(x: u64) {
        while (x > 0) {
            if (x % 2 == 0) { x = x + 1 }
            else { x = x - 2 }
        }
    }

    #[callable]
    fun f2(x: u64) {
        while (x > 0
            && x > 0
            && x > 0
            && x > 0
            && x > 0
            && x > 0
            && x > 0
            && x > 0
            && x > 0
            && x > 0
            && x > 0) {
            if (x % 2 == 0) { x = x + 1 }
            else { x = x - 2 }
        }
    }
}
