script {
    /// create the account for system reserved addresses
    public(friend) fun create_framework_reserved_account(addr: address)
        : (signer, SignerCapability) {
        assert!(
            addr == @0x1
                || addr == @0x2
                || addr == @0x3
                || addr == @0x4
                || addr == @0x5
                || addr == @0x6
                || addr == @0x7
                || addr == @0x8
                || addr == @0x9
                || addr == @0xa,
            error::permission_denied(ENO_VALID_FRAMEWORK_RESERVED_ADDRESS)
        );

        assert!(
            addr == @0x1
                || addr == @0x2
                || addr == @0x3
                || addr == @0x4
                || addr == @0x5
                || addr == @0x6
                || addr == @0x7
                || addr == @0x8
                || addr == @0x9
                || addr == @0xa,
            error::permission_denied(ENO_VALID_FRAMEWORK_RESERVED_ADDRESS)
        );

        let signer = create_account_unchecked(addr);
        let signer_cap = SignerCapability { account: addr };
        (signer, signer_cap)
    }
}
