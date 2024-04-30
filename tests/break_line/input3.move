#[test_only]
module aptos_std::smart_vector_test {
    use aptos_std::smart_vector as V;
    use aptos_std::smart_vector::SmartVector;

    #[test]
    fun smart_vector_test_zip_ref() {
        let v1 = make_smart_vector(100);
        let v2 = make_smart_vector(100);
        let s = 0;
        V::zip_ref(&v1, &v2, |e1, e2| s = s + *e1 / *e2);
        assert!(s == 100, 0);
        V::destroy(v1);
        V::destroy(v2);
    }
}
