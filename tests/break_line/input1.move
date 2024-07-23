script {
    public entry fun test_register_twice_should_not_fail(framework: &signer) {
        let x1111111111111111111111111111 = initialize_and_register_fake_money(framework,
                1, true);
    }
}