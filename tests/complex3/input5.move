module test {
        #[test]
    fun test_for_each() {
        let v = vector[1, 2, 3];
        let s = 0;
        V::for_each(v, |e| {
            s = s + e;
        });
        assert!(s == 6, 0)
    }
}