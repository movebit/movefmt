module test {
        #[test_only]
    public fun resolve_proposal_for_test(
        proposal_id: u64,
        signer_address: address,
        multi_step: bool,
        finish_multi_step_execution: bool
    ): signer acquires ApprovedExecutionHashes, GovernanceResponsbility {
        if (multi_step) {
            let execution_hash = vector::empty<u8>();
            vector::push_back(&mut execution_hash, 1);

            if (finish_multi_step_execution) {
                resolve_multi_step_proposal(proposal_id, signer_address, vector::empty<u8>())
            } else {
                resolve_multi_step_proposal(proposal_id, signer_address, execution_hash)
            }
        } else {
            resolve(proposal_id, signer_address)
        }
    }

        /// Return the current state of a voting delegation of a delegator in a delegation pool.
    public fun calculate_and_update_voting_delegation(
        pool_address: address,
        delegator_address: address
    ): (address, address, u64) acquires DelegationPool, GovernanceRecords {
        assert_partial_governance_voting_enabled(pool_address);
        let vote_delegation = update_and_borrow_mut_delegator_vote_delegation(
            borrow_global<DelegationPool>(pool_address),
            borrow_global_mut<GovernanceRecords>(pool_address),
            delegator_address
        );

        (vote_delegation.voter, vote_delegation.pending_voter, vote_delegation.last_locked_until_secs)
    }
}
