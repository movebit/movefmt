/// test_point: has {'apply', 'exists', 'global'}
spec aptos_std::capability {

    /// Module level specification of capability
    spec module {
        // An uninterpreted function to represent the capability.
        fun cap<CapState>(addr: address): CapState;

        // When the address is equal, the capabilities are equal
        axiom<CapState> forall c1: address, c2: address:
            ( c1 == c2 ==> cap<CapState>(c1) == cap<CapState>(c2) );

	  // Abort if the capability is not exist
        apply CapAbortsIf to *<Feature> except spec_delegates;
    }

    /// Helper specification function to check whether a capability exists at address.
    spec fun spec_has_cap<Feature>(addr: address): bool {
        exists<CapState<Feature>>(addr)
    }

    /// Helper specification function to obtain the delegates of a capability.
    spec fun spec_delegates<Feature>(addr: address): vector<address> {
        global<CapState<Feature>>(addr).delegates
    }
}