module bcs_stream {
        public(friend) fun create_framework_reserved_account(addr: address): (signer, SignerCapability) {}
        
            public native fun create_aggregator<IntElement: copy + drop>(max_value: IntElement): Aggregator<
        IntElement>;


}
