module test_complex_exp {
        spec schema VoteAbortIf {
        voter: &signer;
        stake_pool: address;
        proposal_id: u64;
        should_pass: bool;
        voting_power: u64;

        include VotingGetDelegatedVoterAbortsIf { sign: voter };

        aborts_if spec_proposal_expiration <= locked_until && !exists<timestamp::CurrentTimeMicroseconds>(@aptos_framework);
        let spec_proposal_expiration = voting::spec_get_proposal_expiration_secs<GovernanceProposal>(@aptos_framework, proposal_id);
        let locked_until = global<stake::StakePool>(stake_pool).locked_until_secs;
        let remain_zero_1_cond = (spec_proposal_expiration > locked_until || timestamp::spec_now_seconds() > spec_proposal_expiration);
        let record_key = RecordKey {
            stake_pool,
            proposal_id,
        };
        let entirely_voted = spec_has_entirely_voted(stake_pool, proposal_id, record_key);
        aborts_if !remain_zero_1_cond && !exists<VotingRecords>(@aptos_framework);
        include !remain_zero_1_cond && !entirely_voted ==> GetVotingPowerAbortsIf {
            pool_address: stake_pool
        };

        let staking_config = global<staking_config::StakingConfig>(@aptos_framework);
        let spec_voting_power = spec_get_voting_power(stake_pool, staking_config);
        let voting_records_v2 = borrow_global<VotingRecordsV2>(@aptos_framework);
        let used_voting_power = if (smart_table::spec_contains(voting_records_v2.votes, record_key)) {
            smart_table::spec_get(voting_records_v2.votes, record_key)
        } else {
            0
        };
        aborts_if !remain_zero_1_cond && !entirely_voted && features::spec_partial_governance_voting_enabled() &&
            used_voting_power > 0 && spec_voting_power < used_voting_power;

        let remaining_power = spec_get_remaining_voting_power(stake_pool, proposal_id);
        let real_voting_power =  min(voting_power, remaining_power);
        aborts_if !(real_voting_power > 0);

        aborts_if !exists<VotingRecords>(@aptos_framework);
        let voting_records = global<VotingRecords>(@aptos_framework);


        // verify get_voting_power(stake_pool)
        let allow_validator_set_change = global<staking_config::StakingConfig>(@aptos_framework).allow_validator_set_change;
        let stake_pool_res = global<stake::StakePool>(stake_pool);
        // Two results of get_voting_power(stake_pool) and the third one is zero.

        aborts_if !exists<voting::VotingForum<GovernanceProposal>>(@aptos_framework);
        let voting_forum = global<voting::VotingForum<GovernanceProposal>>(@aptos_framework);
        let proposal = table::spec_get(voting_forum.proposals, proposal_id);
        aborts_if !table::spec_contains(voting_forum.proposals, proposal_id);
        let proposal_expiration = proposal.expiration_secs;
        let locked_until_secs = global<stake::StakePool>(stake_pool).locked_until_secs;
        aborts_if proposal_expiration > locked_until_secs;

        // verify voting::vote
        aborts_if timestamp::now_seconds() > proposal_expiration;
        aborts_if proposal.is_resolved;
        aborts_if !string::spec_internal_check_utf8(voting::IS_MULTI_STEP_PROPOSAL_IN_EXECUTION_KEY);
        let execution_key = utf8(voting::IS_MULTI_STEP_PROPOSAL_IN_EXECUTION_KEY);
        aborts_if simple_map::spec_contains_key(proposal.metadata, execution_key) &&
                  simple_map::spec_get(proposal.metadata, execution_key) != std::bcs::to_bytes(false);
        // Since there are two possibilities for voting_power, the result of the vote is not only related to should_pass,
        // but also to allow_validator_set_change which determines the voting_power
        aborts_if
            if (should_pass) { proposal.yes_votes + real_voting_power > MAX_U128 } else { proposal.no_votes + real_voting_power > MAX_U128 };
        let post post_voting_forum = global<voting::VotingForum<GovernanceProposal>>(@aptos_framework);
        let post post_proposal = table::spec_get(post_voting_forum.proposals, proposal_id);

        aborts_if !string::spec_internal_check_utf8(voting::RESOLVABLE_TIME_METADATA_KEY);
        let key = utf8(voting::RESOLVABLE_TIME_METADATA_KEY);
        ensures simple_map::spec_contains_key(post_proposal.metadata, key);
        ensures simple_map::spec_get(post_proposal.metadata, key) == std::bcs::to_bytes(timestamp::now_seconds());

        aborts_if features::spec_partial_governance_voting_enabled() && used_voting_power + real_voting_power > MAX_U64;
        aborts_if !features::spec_partial_governance_voting_enabled() && table::spec_contains(voting_records.votes, record_key);


        aborts_if !exists<GovernanceEvents>(@aptos_framework);

        // verify voting::get_proposal_state
        let early_resolution_threshold = option::spec_borrow(proposal.early_resolution_vote_threshold);
        let is_voting_period_over = timestamp::spec_now_seconds() > proposal_expiration;

        let new_proposal_yes_votes_0 = proposal.yes_votes + real_voting_power;
        let can_be_resolved_early_0 = option::spec_is_some(proposal.early_resolution_vote_threshold) &&
                                    (new_proposal_yes_votes_0 >= early_resolution_threshold ||
                                     proposal.no_votes >= early_resolution_threshold);
        let is_voting_closed_0 = is_voting_period_over || can_be_resolved_early_0;
        let proposal_state_successed_0 = is_voting_closed_0 && new_proposal_yes_votes_0 > proposal.no_votes &&
                                         new_proposal_yes_votes_0 + proposal.no_votes >= proposal.min_vote_threshold;
        let new_proposal_no_votes_0 = proposal.no_votes + real_voting_power;
        let can_be_resolved_early_1 = option::spec_is_some(proposal.early_resolution_vote_threshold) &&
                                    (proposal.yes_votes >= early_resolution_threshold ||
                                     new_proposal_no_votes_0 >= early_resolution_threshold);
        let is_voting_closed_1 = is_voting_period_over || can_be_resolved_early_1;
        let proposal_state_successed_1 = is_voting_closed_1 && proposal.yes_votes > new_proposal_no_votes_0 &&
                                         proposal.yes_votes + new_proposal_no_votes_0 >= proposal.min_vote_threshold;
        let new_proposal_yes_votes_1 = proposal.yes_votes + real_voting_power;
        let can_be_resolved_early_2 = option::spec_is_some(proposal.early_resolution_vote_threshold) &&
                                    (new_proposal_yes_votes_1 >= early_resolution_threshold ||
                                     proposal.no_votes >= early_resolution_threshold);
        let is_voting_closed_2 = is_voting_period_over || can_be_resolved_early_2;
        let proposal_state_successed_2 = is_voting_closed_2 && new_proposal_yes_votes_1 > proposal.no_votes &&
                                         new_proposal_yes_votes_1 + proposal.no_votes >= proposal.min_vote_threshold;
        let new_proposal_no_votes_1 = proposal.no_votes + real_voting_power;
        let can_be_resolved_early_3 = option::spec_is_some(proposal.early_resolution_vote_threshold) &&
                                    (proposal.yes_votes >= early_resolution_threshold ||
                                     new_proposal_no_votes_1 >= early_resolution_threshold);
        let is_voting_closed_3 = is_voting_period_over || can_be_resolved_early_3;
        let proposal_state_successed_3 = is_voting_closed_3 && proposal.yes_votes > new_proposal_no_votes_1 &&
                                         proposal.yes_votes + new_proposal_no_votes_1 >= proposal.min_vote_threshold;
        // post state
        let post can_be_resolved_early = option::spec_is_some(proposal.early_resolution_vote_threshold) &&
                                    (post_proposal.yes_votes >= early_resolution_threshold ||
                                     post_proposal.no_votes >= early_resolution_threshold);
        let post is_voting_closed = is_voting_period_over || can_be_resolved_early;
        let post proposal_state_successed = is_voting_closed && post_proposal.yes_votes > post_proposal.no_votes &&
                                         post_proposal.yes_votes + post_proposal.no_votes >= proposal.min_vote_threshold;
        // verify add_approved_script_hash(proposal_id)
        let execution_hash = proposal.execution_hash;
        let post post_approved_hashes = global<ApprovedExecutionHashes>(@aptos_framework);

        // Due to the complexity of the success state, the validation of 'borrow_global_mut<ApprovedExecutionHashes>(@aptos_framework);' is discussed in four cases.
        /// [high-level-req-3]
        aborts_if
            if (should_pass) {
                proposal_state_successed_0 && !exists<ApprovedExecutionHashes>(@aptos_framework)
            } else {
                proposal_state_successed_1 && !exists<ApprovedExecutionHashes>(@aptos_framework)
            };
        aborts_if
            if (should_pass) {
                proposal_state_successed_2 && !exists<ApprovedExecutionHashes>(@aptos_framework)
            } else {
                proposal_state_successed_3 && !exists<ApprovedExecutionHashes>(@aptos_framework)
            };
        ensures proposal_state_successed ==> simple_map::spec_contains_key(post_approved_hashes.hashes, proposal_id) &&
                                             simple_map::spec_get(post_approved_hashes.hashes, proposal_id) == execution_hash;

        aborts_if features::spec_partial_governance_voting_enabled() && !exists<VotingRecordsV2>(@aptos_framework);
    }
}
