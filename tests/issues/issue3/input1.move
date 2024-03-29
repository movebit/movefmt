module TestFunFormat {
    struct S has store {}
    struct R has store {}
    struct T has store {}
    struct G<T> has store {}

    fun f1() acquires S {
    }

    fun f2() acquires S(*) {
    }

    fun f3() acquires 0x42::*::* {
    }

    fun f4() acquires 0x42::m::* {
    }

    fun f5() acquires *(*) {
    }

    fun f6() acquires *(0x42) {
    }

    fun f7(a: address) acquires *(a) {
    }

    fun f8(x: u64) acquires *(make_up_address(x)) {
    }
        
    fun f9() acquires LongName1, LongName2,LongName3, LongName4,LongName5, LongName6, LongName7, LongName8, LongName9, LongName10 {
    }        

}
