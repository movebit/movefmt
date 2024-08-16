#[fmt::skip]
module 0xc0ffee::m {
    public fun test() {
        if ({let x = foo(); !x}) {
            bar();
        }
    }
}