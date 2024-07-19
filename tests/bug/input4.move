script {
    entry fun store_entry_function_payload_from_native_txn_context<T1, T2, T3>(
        _s: &signer, arg0: u64, arg1: bool
    ) acquires TransactionContextStore {
        let store = borrow_global_mut<TransactionContextStore>(@admin);
        let payload_opt = transaction_context::entry_function_payload();
        if (option::is_some(&payload_opt)) {
            // `store` is one kind of abilities
            let payload = transaction_context::borrow(&payload_opt);store.account_address = transaction_context::account_address(
                payload
            );
            
            store.module_name = transaction_context::module_name(payload);store.function_name =
                 transaction_context::function_name(payload);store.type_arg_names = transaction_context::type_arg_names(
                payload
            );store.args = transaction_context::args(payload);

            // Check that the arguments are correct and can be parsed using `from_bcs`.
            assert!(arg0 == from_bcs::to_u64(*vector::borrow(&store.args, 0)), 11);
            assert!(arg1 == from_bcs::to_bool(*vector::borrow(&store.args, 1)), 12);
            // Check that the type argument names are correct and matched to `type_info::type_name`.
            assert!(
                store.type_arg_names
                    == vector[
                        type_info::type_name<T1>(),
                        type_info::type_name<T2>(),
                        type_info::type_name<T3>()],
                13,
            );
        }
    }
}