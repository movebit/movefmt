/// test_point: has {'use', 'include'}
spec aptos_framework::block {
    spec module {
        use aptos_framework::chain_status;
        // After genesis, `BlockResource` exist.
        invariant [suspendable] chain_status::is_operating() ==>
            exists<BlockResource>(@aptos_framework);
    }

    spec BlockResource {
        invariant epoch_interval > 0;
    }

    spec block_prologue {
        use aptos_framework::chain_status;
        use aptos_framework::coin::CoinInfo;
        use aptos_framework::aptos_coin::AptosCoin;
        use aptos_framework::transaction_fee;
        use aptos_framework::staking_config;

        pragma verify_duration_estimate = 120; // TODO: set because of timeout (property proved)
        requires chain_status::is_operating();
        include transaction_fee::RequiresCollectedFeesPerValueLeqBlockAptosSupply;
        include staking_config::StakingRewardsConfigRequirement;
        aborts_if false;
    }
}
